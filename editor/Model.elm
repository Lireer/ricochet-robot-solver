module Model exposing (..)

import AllDict exposing (AllDict)
import Mouse exposing (Position)
import BoardConfig exposing (boardSizeInFields)


type alias Model =
    { board : Board
    , objects : Positions
    , drag : Maybe Drag
    , error: Maybe String
    }


type alias JsonError =
    String


type alias Drag =
    { -- start is needed to make sure that the robot isn't jumped to the mouse position but instead is smoothly dragged
      start :
        Position
        -- current - start is the offset that needs to be applied to the dragged robot
    , current : Position
    , object : Object
    }


type Object
    = Robot RobotColor
    | Target Target


colOrd : RobotColor -> number
colOrd c =
    case c of
        Red ->
            0

        Green ->
            1

        Blue ->
            2

        Yellow ->
            3


objOrd : Object -> number
objOrd obj =
    case obj of
        Robot c ->
            100 + (colOrd c)

        Target Spiral ->
            0

        Target (Circle c) ->
            10 + (colOrd c)

        Target (Square c) ->
            20 + (colOrd c)

        Target (Triangle c) ->
            30 + (colOrd c)

        Target (Hexagon c) ->
            40 + (colOrd c)


type Target
    = Spiral
    | Circle RobotColor
    | Triangle RobotColor
    | Square RobotColor
    | Hexagon RobotColor


type alias Board =
    List Row


type alias Row =
    List Field


type alias Field =
    { bottom : Bool
    , right : Bool
    }


type RobotColor
    = Red
    | Green
    | Blue
    | Yellow


type alias Positions =
    AllDict Object ( Int, Int ) Int


field : Field
field =
    { bottom = False, right = False }


model : Model
model =
    let
        most =
            List.repeat 15 (List.append (List.repeat 15 field) [ { field | right = True } ])

        last =
            List.append (List.repeat 15 { field | bottom = True }) [ { field | bottom = True, right = True } ]

        targets =
            [ Spiral
            , Circle Red
            , Circle Green
            , Circle Blue
            , Circle Yellow
            , Triangle Red
            , Triangle Green
            , Triangle Blue
            , Triangle Yellow
            , Square Red
            , Square Green
            , Square Blue
            , Square Yellow
            , Hexagon Red
            , Hexagon Green
            , Hexagon Blue
            , Hexagon Yellow
            ]
                |> List.indexedMap (\i v -> ( Target v, ( i % boardSizeInFields, i // boardSizeInFields ) ))

        robots =
            [ ( Robot Red, ( 1, 3 ) ), ( Robot Green, ( 15, 12 ) ), ( Robot Blue, ( 13, 8 ) ), ( Robot Yellow, ( 6, 6 ) ) ]
    in
        { board =
            List.append most [ last ]
                |> toggleBoardWall 7 6 Bottom
                |> toggleBoardWall 8 6 Bottom
                |> toggleBoardWall 7 8 Bottom
                |> toggleBoardWall 8 8 Bottom
                |> toggleBoardWall 6 7 Right
                |> toggleBoardWall 6 8 Right
                |> toggleBoardWall 8 7 Right
                |> toggleBoardWall 8 8 Right
        , drag = Nothing
        , objects = AllDict.fromList objOrd (List.append targets robots)
        , error = Nothing
        }


type Msg
    = ToggleWall Int Int Wall
    | DragStart Position Object
    | DragAt Position
    | DragEnd Position
    | NewJson String


type Wall
    = Right
    | Bottom


toggleBoardWall : Int -> Int -> Wall -> Board -> Board
toggleBoardWall x y wall board =
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
