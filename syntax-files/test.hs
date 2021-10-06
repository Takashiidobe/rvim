import Data.Maybe
import Numeric
import Data.Char

data Person = Person { firstName :: String, lastName :: String } deriving Show

person = Person "Erik" "Salaj"

nothing = Nothing :: Maybe String
just = Just "Hello, world!" :: Maybe String

main = do
    print person
    print Person { firstName = "Erik", lastName = "Salaj" }
    print $ firstName person
    print $ lastName person

    print $ showInt 123 ""
    print $ showHex 123 ""
    print $ showOct 123 ""
    print $ showIntAtBase 2 intToDigit 123 ""

    print $ showFloat 123.456 ""
    print $ showEFloat (Just 2) 123.456 ""
    print $ showFFloat (Just 2) 123.456 ""
    print $ showGFloat (Just 2) 123.456 ""
    print $ floatToDigits 10 123.456
    print $ floatToDigits 16 123.456

    print nothing
    print just

    print $ isNothing nothing
    print $ isNothing just

    print $ isJust nothing
    print $ isJust just

    case nothing of
        Nothing -> print "Nothing"
        Just x -> print x

    case just of
        Nothing -> print "Nothing"
        Just x -> print x

    print $ maybe "Default" (map toUpper) nothing
    print $ maybe "Default" (map toUpper) just

    print $ fromJust just

    print $ fromMaybe "Default" nothing
    print $ fromMaybe "Default" just

    print $ listToMaybe ([] :: [Int])
    print $ listToMaybe [1, 2, 3]

    print $ maybeToList nothing
    print $ maybeToList just

    print $ catMaybes [nothing, just]

    print $ mapMaybe (\_ -> (Nothing :: Maybe Int)) [1, 2, 3]
    print $ mapMaybe (\x -> Just x) [1, 2, 3]
