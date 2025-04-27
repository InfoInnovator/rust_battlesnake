# rocket-server

This crate provides the necessary API endpoints for the main battlesnake game.
Its basically the main [start-snake-rust](https://github.com/BattlesnakeOfficial/starter-snake-rust) template but without the game-types.
Its modified in way that it uses the snake from the `battlesnakes` crate using the name provided in the JSON body of the move request, so you can use multiple snakes from a single webserver.

## Endpoints
### "/"

This endpoint is supposed to provide the main info about the snake (author, color, head, tail), but due to a change in the Battlesnake API the snake name is no longer provided to this route.
Because of this, all snakes currently have the same properties.

### "/start"

This returns a HTTP status code OK, just like the template.

### "/end"

This also returns a HTTP status code OK.

### "/move"

The move request is provided with the full game info, so you can extract the name of the called snake ('you') from it and use it to call the right snake.

It also saves the full game state into a JSON file. This is done for the last ten turns and can be used by `xtask` and the `runa` to create new test scenarios.
The state kept by rocket can only access a mutex so this feature should only be used in development because multiple concurrent snakes will write to the same file and mix up the game states.