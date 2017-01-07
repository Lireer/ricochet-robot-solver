module BoardConfig exposing (..)


fieldSize : Float
fieldSize =
    25


boardSizeInFields : Int
boardSizeInFields =
    16


boardSize : Float
boardSize =
    fieldSize * toFloat boardSizeInFields


boardOffset : Float
boardOffset =
    5
