-- Read more about this program in the official Elm guide:
-- https://guide.elm-lang.org/architecture/user_input/buttons.html


module Main exposing (..)

import Html exposing (beginnerProgram, div, button)
import Graphics.Render exposing (Point, centered, text, Form, group, solid, circle, ellipse, polygon, filledAndBordered, position, svg, rectangle, filled, angle, fontColor, segment, solidLine, onClick)
import Color exposing (rgb)


main : Program Never Model Msg
main =
    beginnerProgram { model = model, view = view, update = update }


fieldSize : Float
fieldSize =
    25


boardSizeInFields : Int
boardSizeInFields =
    16


boardSize : Float
boardSize =
    fieldSize * toFloat boardSizeInFields


viewRow : Int -> List Field -> List (Form Msg)
viewRow y row =
    List.concat (List.indexedMap (viewField y) row)


boardOffset : Float
boardOffset =
    5


indexToPosition : Int -> Float
indexToPosition i =
    (toFloat i) * fieldSize + boardOffset


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
    in
        onClick (ToggleWall x y wall)
            (drawLine ( x1, y1 )
                ( x2, y2 )
                (if fill then
                    Color.black
                 else
                    Color.lightGray
                )
                boardOffset
            )


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


view : Model -> Html.Html Msg
view model =
    svg 0
        0
        (boardSize
            + 10
        )
        (boardSize
            + 10
        )
        (group
            (List.concat
                (List.indexedMap
                    viewRow
                    model.board
                )
            )
         --[
         -- drawRectangle boardSize boardSize ( boardSize / 2, boardSize / 2 ) Color.lightGray
         --, drawEllipse ( 30, 30 )
         --, drawCircle ( boardSize - 30, 30 )
         --, drawEllipse ( boardSize - 30, boardSize - 30 )
         --, drawCircle ( 30, boardSize - 30 )
         --, drawPolygon ( 100, 100 ) (degrees 210) Color.green
         --, drawPolygon ( 150, 100 ) (degrees 160) Color.yellow
         --, drawForm ( 1000, 200 ) (degrees 10)
         --, drawText "Demo text" 60 ( boardSize / 2, boardSize / 2 ) Color.black
         --]
        )


drawForm : Point -> Float -> Form msg
drawForm pos rotation =
    group
        [ drawRectangle 300 150 ( 0, 0 ) Color.blue
        , drawText "A separate form" 20 ( 0, 0 ) Color.yellow
        , drawCircle ( 0, 40 )
        ]
        |> angle rotation
        |> position pos


drawPolygon : Point -> Float -> Color.Color -> Form msg
drawPolygon pos rotation color =
    polygon [ ( 0, 0 ), ( 10, -10 ), ( 10, -20 ), ( -10, -20 ), ( -10, -10 ) ]
        |> filled (solid <| color)
        |> angle rotation
        |> position pos


drawRectangle : Float -> Float -> Point -> Color.Color -> Form msg
drawRectangle width height pos color =
    rectangle width height
        |> filled (solid <| color)
        |> position pos


drawLine : Point -> Point -> Color.Color -> Float -> Form msg
drawLine start end color width =
    segment start end
        |> solidLine width (solid color)


drawEllipse : Point -> Form msg
drawEllipse pos =
    ellipse 10 20
        |> filledAndBordered (solid <| rgb 0 0 255)
            5
            (solid <| rgb 0 0 0)
        |> position pos


drawCircle : Point -> Form msg
drawCircle pos =
    circle 20
        |> filledAndBordered (solid <| rgb 255 0 0)
            5
            (solid <| rgb 0 0 0)
        |> position pos


drawText : String -> Int -> Point -> Color.Color -> Form msg
drawText textContent textSize pos color =
    text textSize textContent
        |> fontColor color
        |> centered
        |> position pos



-- MODEL


type alias Model =
    { board : Board
    , positions : RobotPositions
    }


type alias Board =
    List Row


type alias Row =
    List Field


type alias Field =
    { bottom : Bool
    , right : Bool
    }


type alias RobotPositions =
    List ( Int, Int )


field : Field
field =
    { bottom = False, right = False }


model : Model
model =
    let
        most =
            List.repeat 15 (List.append (List.repeat 15 { bottom = False, right = False }) [ { bottom = False, right = True } ])

        last =
            List.append (List.repeat 15 { bottom = True, right = False }) [ { bottom = True, right = True } ]
    in
        { board = List.append most [ last ]
        , positions = [ ( 1, 1 ), ( 15, 12 ), ( 13, 8 ), ( 6, 6 ) ]
        }


type Msg
    = ToggleWall Int Int Wall


type Wall
    = Right
    | Bottom


update : Msg -> Model -> Model
update msg model =
    case msg of
        ToggleWall x y wall ->
            { model | board = toggleBoardWall model.board x y wall }


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
