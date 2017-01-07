-- A ricochet robots visualizer and map editor


module Main exposing (..)

import Html exposing (program, div, button)
import Mouse exposing (Position)
import AllDict exposing (AllDict)
import Model exposing (..)
import BoardConfig exposing (..)
import View exposing (..)


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
        ToggleWall x y wall ->
            ( { model | board = toggleBoardWall model.board x y wall }, Cmd.none )

        -- initialize a drag with the current mouse position
        DragStart pos obj ->
            ( { model | drag = Just { start = pos, current = pos, object = obj } }, Cmd.none )

        -- update the visual position of the robot while being dragged
        DragAt pos ->
            ( { model | drag = Maybe.map (\drag -> { drag | current = pos }) model.drag }, Cmd.none )

        -- when the robot is dropped, move it to the target
        DragEnd pos ->
            ( { model | drag = Nothing, objects = Maybe.withDefault model.objects (Maybe.map (updatePosition model.objects) model.drag) }, Cmd.none )


collides : ( Object, ( Int, Int ) ) -> ( Object, ( Int, Int ) ) -> Bool
collides ( a, ap ) ( b, bp ) =
    case ( a, b ) of
        ( Robot _, Robot _ ) ->
            ap == bp

        ( Target _, Target _ ) ->
            ap == bp

        _ ->
            False


updateObjectPosition : Drag -> List ( Object, ( Int, Int ) ) -> ( Int, Int ) -> ( Int, Int )
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


updatePosition : Positions -> Drag -> Positions
updatePosition positions drag =
    positions
        |> AllDict.update drag.object (Maybe.map (updateObjectPosition drag (AllDict.toList positions)))


{-| Calculate the new grid position from the drag position and the old position.
In case the new grid position is outside the grid, snap back to the old position
-}
xy2pos : Drag -> ( Int, Int ) -> ( Int, Int )
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


toggleBoardWall : Board -> Int -> Int -> Wall -> Board
toggleBoardWall board x y wall =
    List.indexedMap
        (\y_i row ->
            (if y_i == y then
                (List.indexedMap
                    (\x_i field ->
                        (if x_i == x then
                            (toggleFieldWall field wall)
                         else
                            field
                        )
                    )
                    row
                )
             else
                row
            )
        )
        board


toggleFieldWall : Field -> Wall -> Field
toggleFieldWall field wall =
    case wall of
        Right ->
            { field | right = not field.right }

        Bottom ->
            { field | bottom = not field.bottom }



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    case model.drag of
        Nothing ->
            Sub.none

        Just _ ->
            Sub.batch [ Mouse.moves DragAt, Mouse.ups DragEnd ]
