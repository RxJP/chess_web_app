let piecesContainer = undefined;
let boardImage = undefined;
let boardPos = [0.0, 0.0];
let boardSize = [0.0, 0.0];

const init_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
let game_idx = 0;
let move_idx = 0;

const Pieces = {
    WHITE_KING:     'svgs/pieces/King-White.svg',
    WHITE_QUEEN:    'svgs/pieces/Queen-White.svg',
    WHITE_ROOK:     'svgs/pieces/Rook-White.svg',
    WHITE_BISHOP:   'svgs/pieces/Bishop-White.svg',
    WHITE_KNIGHT:   'svgs/pieces/Knight-White.svg',
    WHITE_PAWN:     'svgs/pieces/Pawn-White.svg',
    BLACK_KING:     'svgs/pieces/King-Black.svg',
    BLACK_QUEEN:    'svgs/pieces/Queen-Black.svg',
    BLACK_ROOK:     'svgs/pieces/Rook-Black.svg',
    BLACK_BISHOP:   'svgs/pieces/Bishop-Black.svg',
    BLACK_KNIGHT:   'svgs/pieces/Knight-Black.svg',
    BLACK_PAWN:     'svgs/pieces/Pawn-Black.svg',
};

let board = [
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined],
    [undefined, undefined, undefined, undefined, undefined, undefined, undefined, undefined]
];

function createPiece(pos, piecePath) {
    const row = pos[0];
    const col = pos[1];
    const piece = document.createElement('img');
    piece.src = piecePath;
    piece.alt = 'Chess Piece';
    piece.className = 'chess-piece';

    const pxpos = getPiecePosition(row, col);
    piece.style.left = `${pxpos[0]}px`
    piece.style.top = `${pxpos[1]}px`;
    piece.style.width = `${boardSize[0]/8}px`;
    piece.style.height = `${boardSize[1]/8}px`;

    if(board[row][col])
        board[row][col].remove();
    board[row][col] = piece;
    piecesContainer.appendChild(piece);
}
function movePiece(from, to) {
    const r1 = from[0], c1 = from[1];
    const r2 = to[0], c2 = to[1];
    if(board[r1][c1]) {
        if(board[r2][c2])
            board[r2][c2].remove();
        board[r2][c2] = board[r1][c1];
        updatePiecePos(to);
        board[r1][c1] = undefined;
    }
}
function getPiecePosition(row, col) {
    return [(col / 8) * boardSize[0] + boardPos[0], (row / 8) * boardSize[1] + boardPos[1]];
}
function updatePiecePos(pos) {
    const i = pos[0], j = pos[1];
    const pxpos = getPiecePosition(i, j);
    board[i][j].style.left = `${pxpos[0]}px`;
    board[i][j].style.top = `${pxpos[1]}px`;
    board[i][j].style.width = `${boardSize[0]/8}px`;
    board[i][j].style.height = `${boardSize[1]/8}px`;
}
function resetBoard() {
    for(let i = 0; i < piecesContainer.children.length; i++)
        piecesContainer.children.item(i).remove();
    parseFEN(init_fen);
}

function onResizeHandler() {
    const rect = boardImage.getBoundingClientRect();
    if(boardPos[0] === rect.left && boardPos[1] === rect.top && boardSize[0] === rect.width && boardSize[1] === rect.height)
        return;
    boardPos[0] = rect.left;
    boardPos[1] = rect.top;
    boardSize[0] = rect.width;
    boardSize[1] = rect.height;

    console.log(rect);
    // console.log(`Pos: ${boardPos}\t\t Size: ${boardSize}`);

    for(let i = 0; i < 8; i++) {
        for(let j = 0; j < 8; j++) {
            if(board[i][j] === undefined)
                continue;
            updatePiecePos([i,j]);
        }
    }
}

function playMove(move) {
    movePiece(move.from, move.to);
    if(move.is_long_castle) {
        movePiece([move.from[0], 0], [move.from[0], 3]);
    }
    else if(move.is_short_castle) {
        movePiece([move.from[0], 7], [move.from[0], 5]);
    }
    else if(move.is_en_passant) {
        board[move.from[0]][move.to[1]].remove();
        board[move.from[0]][move.to[1]] = undefined;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    piecesContainer = document.getElementById('pieces-container');

    boardImage = document.getElementById('chessboard');

    boardImage.addEventListener('load', () => {
        resetBoard();
    });
    setInterval(() => {
        if(!games) return;
        let game = games[game_idx];
        if(move_idx === game.length) {
            move_idx = 0;
            resetBoard();
            game_idx++;
            if(game_idx === games.length)
                game_idx = 0;
        }
        let move = game[move_idx++];
        playMove(move);
    }, 2000);
    setInterval(onResizeHandler, 200);

    document.getElementById("play-bot-button").addEventListener('click', () => {
        window.location.href = '/bot_play';
    });

    const account_button = document.getElementById('account-button');
    if(userData.is_authenticated) {
        //todo add a user profile menu
        const menu = document.getElementById('drop-down-menu');

        account_button.addEventListener('click', () => {
            menu.style.display = menu.style.display === 'block' ? 'none' : 'block';

            const rect = account_button.getBoundingClientRect();
            menu.style.top = `${rect.bottom + window.scrollY}px`;
            menu.style.left = `${rect.left + window.scrollX}px`;
        });

        document.addEventListener('click', (event) => {
            if (!menu.contains(event.target) && event.target !== account_button) {
                menu.style.display = 'none';
            }
        });

        document.getElementById("log-out-button").addEventListener('click', async () => {
            let response = await fetch("/logout", {
                method: 'POST',
                headers: {'Content-Type': 'application/json'}
            });
            if (response.ok) {
                window.location.reload();
            } else {
                console.error('Form submission failed:', response.status, response.statusText);
            }
        });
    }
    else {
        account_button.addEventListener('click', () => {
            window.location.href = '/auth';
        });
    }
});

const FENPieces = {
    'K': 'WHITE_KING',
    'Q': 'WHITE_QUEEN',
    'R': 'WHITE_ROOK',
    'B': 'WHITE_BISHOP',
    'N': 'WHITE_KNIGHT',
    'P': 'WHITE_PAWN',
    'k': 'BLACK_KING',
    'q': 'BLACK_QUEEN',
    'r': 'BLACK_ROOK',
    'b': 'BLACK_BISHOP',
    'n': 'BLACK_KNIGHT',
    'p': 'BLACK_PAWN',
};
function parseFEN(fen) {
    const rows = fen.split('/');
    rows.forEach((rowString, rowIndex) => {
        let colIndex = 0;

        for (const char of rowString) {
            if (/[1-8]/.test(char)) {
                colIndex += parseInt(char, 10);
            } else if (FENPieces[char]) {
                const pieceKey = FENPieces[char]; // Get the piece constant
                const piece = Pieces[pieceKey];  // Get the image path from Pieces
                createPiece([rowIndex, colIndex], piece);
                colIndex++;
            }
        }
    });
}