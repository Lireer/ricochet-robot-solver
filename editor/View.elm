module View exposing (..)

import Model exposing (..)
import BoardConfig exposing (..)
import Draw exposing (..)
import Graphics.Render exposing (Point, centered, text, Form, group, solid, circle, ellipse, polygon, filledAndBordered, position, svg, rectangle, filled, angle, fontColor, segment, solidLine, onClick, onMouseDown)
import Color exposing (rgb)
import EveryDict exposing (EveryDict)
import Html exposing (program, div, button)


indexToPosition : Int -> Float
indexToPosition i =
    (toFloat i) * fieldSize + boardOffset


viewRow : Int -> List Field -> List (Form Msg)
viewRow y row =
    row
        |> List.indexedMap (viewField y)
        |> List.concat


viewWall : Bool -> Wall -> Int -> Int -> Form Msg
viewWall fill wall x y =
    let
        ( posx, posy ) =
            ( indexToPosition x, indexToPosition y )

        longer =
            if fill then
                -boardOffset / 2
            else
                boardOffset / 2

        ( x1, y1, x2, y2 ) =
            case wall of
                Right ->
                    ( (posx + fieldSize), (posy - longer), (posx + fieldSize), (posy + fieldSize + longer) )

                Bottom ->
                    ( (posx - longer), (posy + fieldSize), (posx + fieldSize + longer), (posy + fieldSize) )

        color =
            if fill then
                Color.black
            else
                Color.lightGray
    in
        drawLine ( x1, y1 ) ( x2, y2 ) color boardOffset
            |> onClick (ToggleWall x y wall)


viewField : Int -> Int -> Field -> List (Form Msg)
viewField y x field =
    List.concat
        [ [ (viewWall field.right
                Right
                x
                y
            )
          , (viewWall field.bottom
                Bottom
                x
                y
            )
          ]
        , (if x == boardSizeInFields - 1 then
            [ (viewWall field.right
                Right
                -1
                y
              )
            ]
           else
            []
          )
        , (if y == boardSizeInFields - 1 then
            [ (viewWall field.bottom
                Bottom
                x
                -1
              )
            ]
           else
            []
          )
        ]


viewRobots : Maybe Drag -> ( RobotColor, ( Int, Int ) ) -> Form Msg
viewRobots drag ( i, ( x, y ) ) =
    let
        color =
            case i of
                Red ->
                    Color.red

                Green ->
                    Color.green

                Blue ->
                    Color.blue

                Yellow ->
                    Color.yellow

        f =
            \x current start drag ->
                (indexToPosition x)
                    + fieldSize
                    / 2
                    + Maybe.withDefault 0
                        (Maybe.map
                            (\drag ->
                                if drag.object == i then
                                    (toFloat (current drag - start drag))
                                else
                                    0
                            )
                            drag
                        )
    in
        drawCircle
            ( (f x (\drag -> drag.current.x) (\drag -> drag.start.x) drag)
            , (f y (\drag -> drag.current.y) (\drag -> drag.start.y) drag)
            )
            color
            (fieldSize / 3)
            |> onMouseDown (\( x, y ) -> DragStart { x = round x, y = round y } i)


view : Model -> Html.Html Msg
view model =
    svg 0
        0
        (boardSize + 10)
        (boardSize + 10)
        (List.append
            (model.board
                |> List.indexedMap viewRow
                |> List.concat
            )
            (model.positions
                |> EveryDict.toList
                |> List.map (viewRobots model.drag)
            )
            |> group
        )
