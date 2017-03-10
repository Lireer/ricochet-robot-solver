-- A ricochet robots visualizer and map editor


module Main exposing (..)

import Html exposing (program, div, button)
import Mouse exposing (Position)
import AllDict exposing (AllDict)
import Model exposing (Model, Msg, model, Positions, Board, Field, Target, Object)
import BoardConfig exposing (..)
import View exposing (..)
import Json.Decode exposing (decodeString, index, Decoder, bool, array, nullable, int, fail, succeed, list, field)
import Json.Decode.Pipeline exposing (decode, required)
import Bitwise exposing (shiftRightBy)


main : Program Never Model Msg
main =
    program
        { init = ( model, Cmd.none )
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Model.ToggleWall x y wall ->
            ( { model | board = Model.toggleBoardWall x y wall model.board }, Cmd.none )

        -- initialize a drag with the current mouse position
        Model.DragStart pos obj ->
            ( { model | drag = Just { start = pos, current = pos, object = obj } }, Cmd.none )

        -- update the visual position of the robot while being dragged
        Model.DragAt pos ->
            ( { model | drag = Maybe.map (\drag -> { drag | current = pos }) model.drag }, Cmd.none )

        -- when the robot is dropped, move it to the target
        Model.DragEnd pos ->
            ( { model | drag = Nothing, objects = Maybe.withDefault model.objects (Maybe.map (updatePosition model.objects) model.drag) }, Cmd.none )

        Model.NewJson text ->
            ( { model | json = parseJson text }, Cmd.none )

        -- do nothing for now
        Model.LoadJson ->
            ( model, Cmd.none )


parseJson : String -> Result String ( Positions, Board )
parseJson text =
    Result.map2 (,) (parsePositions text) (parseBoard text)


parseBoard : String -> Result String Board
parseBoard text =
    decode Field
        |> required "bottom" bool
        |> required "right" bool
        |> list
        |> list
        |> field "fields"
        |> index 1
        |> flip decodeString text


parsePositions : String -> Result String Positions
parsePositions text =
    Result.map2 AllDict.union (parseTargetPositions text) (parseRobPositions text)


expectCompact : Int -> Positions
expectCompact i =
    let
        r = Bitwise.shiftRightBy i 24
        g = (Bitwise.shiftRightBy i 16) % 8
        b = (Bitwise.shiftRightBy i 8) % 8
        y = i % 8
    in
        AllDict.fromList Model.objOrd [
            (Model.Robot Model.Red, compactPos r),
            (Model.Robot Model.Green, compactPos g),
            (Model.Robot Model.Blue, compactPos b),
            (Model.Robot Model.Yellow, compactPos y)
        ]


compactPos : Int -> (Int, Int)
compactPos i = (Bitwise.shiftRightBy i 4, i % 4)


parseRobPositions : String -> Result String Positions
parseRobPositions text =
    int
        |> field "_field0"
        |> index 0
        |> \d -> Json.Decode.decodeString d text
        |> Result.map expectCompact

collect : List (Result err ok) -> Result err (List ok)
collect list =
    List.foldl
        (\elem aggregate ->
            case ( elem, aggregate ) of
                ( _, Err err ) ->
                    Err err

                ( Err err, _ ) ->
                    Err err

                ( Ok val, Ok aggregate ) ->
                    Ok (val :: aggregate)
        )
        (Ok [])
        list


parseTargetPositions : String -> Result String Positions
parseTargetPositions text =
    parseJsonTargetPositions text
        |> Result.map (List.map (\(x, y) -> (Model.Target x, y)))
        |> Result.map (AllDict.fromList Model.objOrd)


jsonColorToColor : String -> Decoder Model.RobotColor
jsonColorToColor col =
    case col of
        "Red" ->
            succeed Model.Red

        "Green" ->
            succeed Model.Green

        "Blue" ->
            succeed Model.Blue

        "Yellow" ->
            succeed Model.Yellow

        _ ->
            fail ("Bad Color: " ++ col)


parseJsonTargetPositions : String -> Result String (List ( Target, ( Int, Int ) ))
parseJsonTargetPositions text =
    jsonTargetDecoder
        |> list
        |> field "targets"
        |> index 1
        |> flip decodeString text

jsonTargetDecoder : Decoder (Target, (Int, Int))
jsonTargetDecoder =
    let
        pos = Json.Decode.map2 (,)
            (int |> Json.Decode.index 0 |> Json.Decode.index 1)
            (int |> Json.Decode.index 1 |> Json.Decode.index 1)
        spiral = Json.Decode.string
            |> Json.Decode.index 0
            |> Json.Decode.andThen (\a -> if a == "Spiral" then succeed Model.Spiral else fail "expected a spiral")
        color = Json.Decode.string
            |> Json.Decode.field "variant"
            |> Json.Decode.index 0
            |> Json.Decode.andThen jsonColorToColor
        shape = Json.Decode.string
            |> Json.Decode.index 0
            |> Json.Decode.field "fields"
            |> Json.Decode.index 0
            |> Json.Decode.andThen
            (\sha -> case sha of
                "Circle" -> succeed Model.Circle
                "Triangle" -> succeed Model.Triangle
                "Square" -> succeed Model.Square
                "Hexagon" -> succeed Model.Hexagon
                other -> fail other)
            |> Json.Decode.map2 (\color shape -> shape color) color
        target = Json.Decode.oneOf [
            spiral,
            shape
        ]
    in
        Json.Decode.map2 (,) target pos


collides : ( Object, ( Int, Int ) ) -> ( Object, ( Int, Int ) ) -> Bool
collides ( a, ap ) ( b, bp ) =
    case ( a, b ) of
        ( Model.Robot _, Model.Robot _ ) ->
            ap == bp

        ( Model.Target _, Model.Target _ ) ->
            ap == bp

        _ ->
            False


updateObjectPosition : Model.Drag -> List ( Object, ( Int, Int ) ) -> ( Int, Int ) -> ( Int, Int )
updateObjectPosition drag positions pos =
    let
        newpos =
            xy2pos drag pos
    in
        if List.any (collides ( drag.object, newpos )) positions then
            pos
        else
            -- don't move two robots on the same field
            newpos


updatePosition : Positions -> Model.Drag -> Positions
updatePosition positions drag =
    positions
        |> AllDict.update drag.object (Maybe.map (updateObjectPosition drag (AllDict.toList positions)))


{-| Calculate the new grid position from the drag position and the old position.
In case the new grid position is outside the grid, snap back to the old position
-}
xy2pos : Model.Drag -> ( Int, Int ) -> ( Int, Int )
xy2pos drag ( x, y ) =
    let
        newx =
            (drag.current.x - drag.start.x |> toFloat) / fieldSize |> round |> (+) x

        newy =
            (drag.current.y - drag.start.y |> toFloat) / fieldSize |> round |> (+) y
    in
        if newx < 0 || newy < 0 || newx >= boardSizeInFields || newy >= boardSizeInFields then
            ( x, y )
        else
            ( newx, newy )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    case model.drag of
        Nothing ->
            Sub.none

        Just _ ->
            Sub.batch [ Mouse.moves Model.DragAt, Mouse.ups Model.DragEnd ]
