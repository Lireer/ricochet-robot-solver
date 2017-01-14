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
import Array exposing (Array)


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


expectTuple : Array Int -> Decoder ( Int, Int )
expectTuple arr =
    case Maybe.map2 (,) (Array.get 0 arr) (Array.get 1 arr) of
        Just success ->
            succeed success

        Nothing ->
            fail "expected inner position to be a tuple"


parseRobPositions : String -> Result String Positions
parseRobPositions text =
    int
        |> array
        |> Json.Decode.andThen expectTuple
        |> list
        |> field "rob_position"
        |> index 0
        |> flip decodeString text
        |> Result.map (\x -> (AllDict.fromList Model.objOrd (List.map2 (,) [ Model.Robot Model.Red, Model.Robot Model.Green, Model.Robot Model.Blue, Model.Robot Model.Yellow ] x)))


parseTargetPositions : String -> Result String Positions
parseTargetPositions text =
    parseJsonTargetPositions text
        |> Result.andThen (List.map (\( target, pos ) -> (target |> jsonTargetToTarget |> Result.map (\target -> ( target, pos )))))
        |> Result.map (AllDict.fromList Model.objOrd)


jsonTargetToTarget : JsonTarget -> Result String Target
jsonTargetToTarget target =
    case ( target.variant, target.fields ) of
        ( "Spiral", Nothing ) ->
            Ok Model.Spiral

        ( "Circle", Just col ) ->
            Result.map Model.Circle (jsonColorToColor col)

        ( "Square", Just col ) ->
            Result.map Model.Square (jsonColorToColor col)

        ( "Triangle", Just col ) ->
            Result.map Model.Triangle (jsonColorToColor col)

        ( "Hexagon", Just col ) ->
            Result.map Model.Hexagon (jsonColorToColor col)

        ( _, _ ) ->
            Err ("Bad variant: " ++ target.variant)


jsonColorToColor : String -> Result String Model.RobotColor
jsonColorToColor col =
    case col of
        "Red" ->
            Ok Model.Red

        "Green" ->
            Ok Model.Green

        "Blue" ->
            Ok Model.Blue

        "Yellow" ->
            Ok Model.Yellow

        _ ->
            Err ("Bad Color: " ++ col)


f1 : Int -> Int -> Maybe JsonTarget -> Maybe ( JsonTarget, ( Int, Int ) )
f1 y x field =
    field |> Maybe.map (\field -> ( field, ( x, y ) ))


targetGridToPositionList : List (List (Maybe JsonTarget)) -> List ( JsonTarget, ( Int, Int ) )
targetGridToPositionList rows =
    rows
        |> List.indexedMap (\y -> (List.indexedMap (f1 y)))
        |> List.concat
        |> List.filterMap (\x -> x)


parseJsonTargetPositions : String -> Result String (List ( JsonTarget, ( Int, Int ) ))
parseJsonTargetPositions text =
    jsonTargetDecoder
        |> field "target"
        |> list
        |> list
        |> Json.Decode.map targetGridToPositionList
        |> field "fields"
        |> index 1
        |> flip decodeString text


targetDecoder : Decoder Target
targetDecoder =
    fail "bar"


type alias JsonTarget =
    { variant : String, fields : Maybe String }


jsonTargetDecoder : Decoder (Maybe JsonTarget)
jsonTargetDecoder =
    fail "foo"


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
