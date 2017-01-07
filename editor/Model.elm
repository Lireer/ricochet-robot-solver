module Model exposing (..)

import EveryDict exposing (EveryDict)
import Mouse exposing (Position)
import BoardConfig exposing (boardSizeInFields)


type alias Model =
    { board : Board
    , objects : Positions
    , drag : Maybe Drag
    }


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
    EveryDict Object ( Int, Int )


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
        { board = List.append most [ last ]
        , drag = Nothing
        , objects = EveryDict.fromList (List.append targets robots)
        }


type Msg
    = ToggleWall Int Int Wall
    | DragStart Position Object
    | DragAt Position
    | DragEnd Position


type Wall
    = Right
    | Bottom
