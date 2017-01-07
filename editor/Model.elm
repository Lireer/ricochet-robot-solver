module Model exposing (..)

import EveryDict exposing (EveryDict)
import Mouse exposing (Position)


type alias Model =
    { board : Board
    , positions : RobotPositions
    , drag : Maybe Drag
    }


type alias Drag =
    { -- start is needed to make sure that the robot isn't jumped to the mouse position but instead is smoothly dragged
      start :
        Position
        -- current - start is the offset that needs to be applied to the dragged robot
    , current : Position
    , object : RobotColor
    }


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


type alias RobotPositions =
    EveryDict RobotColor ( Int, Int )


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
    in
        { board = List.append most [ last ]
        , positions = [ ( Red, ( 1, 1 ) ), ( Green, ( 15, 12 ) ), ( Blue, ( 13, 8 ) ), ( Yellow, ( 6, 6 ) ) ] |> EveryDict.fromList
        , drag = Nothing
        }


type Msg
    = ToggleWall Int Int Wall
    | DragStart Position RobotColor
    | DragAt Position
    | DragEnd Position


type Wall
    = Right
    | Bottom
