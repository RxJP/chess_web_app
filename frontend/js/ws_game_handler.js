let pieceMovesMap = {};
let allMoves = [];
let selectedPiece;

let gameInstance;

function highlightMoveSquare(pos, move) {
    let highlight = document.createElement("div");
    highlight.className = 'highlight-piece';

    const pxpos = getAbsolutePiecePosition(pos[0], pos[1]);
    highlight.style.left = `${pxpos[0]}px`;
    highlight.style.top = `${pxpos[1]}px`;
    highlight.style.width = `${boardSize[0] / 8}px`;
    highlight.style.height = `${boardSize[1] / 8}px`;

    // console.log(pos);
    // console.log(pxpos);

    if (move) {
        highlight.addEventListener('click', () => {
            playMove(move);
            clearPreviousHighlights();
            allMoves = [];
            pieceMovesMap = {};
            selectedPiece = null;
            //send move through socket
            gameInstance.socket.send(JSON.stringify({MovePlayed: move}));
        });
    }

    piecesContainer.append(highlight);
}
function clearPreviousHighlights() {
    const previousHighlights = piecesContainer.getElementsByClassName('highlight-piece');
    while (previousHighlights.length > 0) { previousHighlights.item(0).remove(); }
}

function handlePieceClick(piece) {
    if (selectedPiece && selectedPiece.id === piece.id) {
        selectedPiece = null;
        clearPreviousHighlights();
        return;
    }
    else {
        clearPreviousHighlights();
    }
    selectedPiece = piece;

    if (!pieceMovesMap[piece.id])
        return;
    let moves = pieceMovesMap[piece.id];
    for (let i = 0; i < moves.length; i++) {
        highlightMoveSquare(moves[i].to, moves[i]);
    }
}

function createBotGame() {
    let game_object = {
        is_game_valid: true,
    };
    const socket = new WebSocket('/ws_bot_play');

    socket.addEventListener('open', () => {
        console.log('WebSocket connection established.');
        socket.send(JSON.stringify("Connected"));
    });

    socket.addEventListener('message', (event) => {
        console.log('Message from server:', event.data);

        const data = JSON.parse(event.data);
        let array;
        if ("error" in data) {
            console.error(data.error);
        }
        else if ("is_valid" in data) {
            //last played move was valid
        }
        else if ("game_status" in data) {
            if (data.game_status === "stalemate") {

            }
            else if (data.game_status === "win") {

            }
            else if (data.game_status === "lose") {

            }
        }
        else if ("opp_move" in data) {
            playMove(data.opp_move);
            array = data.legal_moves;
        }
        else if (Array.isArray(data)) {
            array = data;
        }
        if (array) {
            allMoves = array;
            pieceMovesMap = {};
            for (let i = 0; i < array.length; i++) {
                let piece = board[array[i].from[0]][array[i].from[1]];
                // console.log(piece);
                if (!Object.hasOwn(pieceMovesMap, piece.id))
                    pieceMovesMap[piece.id] = [];
                pieceMovesMap[piece.id].push(array[i]);
            }
        }
    });

    socket.addEventListener('error', (error) => {
        console.error('WebSocket error:', error);
    });
    socket.addEventListener('close', (event) => {
        console.log('WebSocket connection closed:', event.code, event.reason);
        game_object.is_game_valid = false;
    });

    setInterval(() => { socket.send(JSON.stringify("KeepAlive")); }, 25000);

    game_object.socket = socket;

    gameInstance = game_object;
}
