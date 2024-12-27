let piecesContainer = undefined;
let boardImage = undefined;
let boardPos = [0.0, 0.0];
let boardSize = [0.0, 0.0];

const init_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

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

let pieceCountUID = 0;
function createPiece(pos, piecePath) {
    const row = pos[0];
    const col = pos[1];
    const piece = document.createElement('img');
    piece.src = piecePath;
    piece.alt = 'Chess Piece';
    piece.className = 'chess-piece';
    piece.id = `${pieceCountUID++}`;

    const pxpos = getAbsolutePiecePosition(row, col);
    piece.style.left = `${pxpos[0]}px`
    piece.style.top = `${pxpos[1]}px`;
    piece.style.width = `${boardSize[0]/8}px`;
    piece.style.height = `${boardSize[1]/8}px`;

    if(board[row][col])
        board[row][col].remove();
    board[row][col] = piece;
    piecesContainer.appendChild(piece);

    if (piecePath.includes("White")) {
        piece.addEventListener('click', () => {
            handlePieceClick(piece);
        });
    }
}
function movePiece(from, to) {
    const r1 = from[0], c1 = from[1];
    const r2 = to[0], c2 = to[1];
    if(board[r1][c1]) {
        if(board[r2][c2])
            board[r2][c2].remove();
        board[r2][c2] = board[r1][c1];
        updateAbsolutePiecePos(to);
        board[r1][c1] = undefined;
    }
}
function getAbsolutePiecePosition(row, col) {
    return [(col / 8) * boardSize[0] + boardPos[0], (row / 8) * boardSize[1] + boardPos[1]];
}
function updateAbsolutePiecePos(pos) {
    const i = pos[0], j = pos[1];
    const pxpos = getAbsolutePiecePosition(i, j);
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

// Function to get the color of a piece from its position
function getPieceColor(position) {
    const [i, j] = position;
    if (!board[i][j] || !board[i][j].src) return null;
    return board[i][j].src.includes("White") ? "WHITE" : "BLACK";
}

// Helper function to get the piece type from Pieces object
function getPieceType(color, pieceName) {
    if (!pieceName || pieceName === "None") return null;
    return Pieces[`${color}_${pieceName.toUpperCase()}`];
}

function playMove(move) {
    const [fromRow, fromCol] = move.from;
    const [toRow, toCol] = move.to;

    const pieceColor = getPieceColor(move.from);

    //castling
    if (Math.abs(fromCol - toCol) === 2 && board[fromRow][fromCol].src.includes("King")) {
        if (toCol > fromCol) {
            // Short castle
            movePiece([fromRow, 7], [fromRow, 5]);
        } else {
            // Long castle
            movePiece([fromRow, 0], [fromRow, 3]);
        }
    }

    //en passant
    if (board[fromRow][fromCol].src.includes("Pawn") &&
        fromCol !== toCol &&
        !board[toRow][toCol]) {
        board[fromRow][toCol].remove();
        board[fromRow][toCol] = undefined;
    }

    //normal move
    movePiece(move.from, move.to);

    //promotion
    if (move.promotion_piece && move.promotion_piece !== "None") {
        const pieceType = getPieceType(pieceColor, move.promotion_piece);
        if (pieceType) {
            createPiece(move.to, pieceType);
        }
    }
}

function onResizeHandler() {
    const rect = boardImage.getBoundingClientRect();
    if(boardPos[0] === rect.left && boardPos[1] === rect.top && boardSize[0] === rect.width && boardSize[1] === rect.height)
        return;
    boardPos[0] = rect.left;
    boardPos[1] = rect.top;
    boardSize[0] = rect.width;
    boardSize[1] = rect.height;

    // console.log(rect);
    // console.log(`Pos: ${boardPos}\t\t Size: ${boardSize}`);

    for(let i = 0; i < 8; i++) {
        for(let j = 0; j < 8; j++) {
            if(board[i][j] === undefined)
                continue;
            updateAbsolutePiecePos([i,j]);
        }
    }
}

document.addEventListener('DOMContentLoaded', () => {
    piecesContainer = document.getElementById('pieces-container');
    boardImage = document.getElementById('chessboard');

    boardImage.addEventListener('load', () => {
        resetBoard();
        createBotGame();
    });

    setInterval(onResizeHandler, 200);

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
                resignGame();
                window.location.href = '/';
            } else {
                console.error('Form submission failed:', response.status, response.statusText);
            }
        });
    }
    else {
        userData.username = "Guest";
        account_button.addEventListener('click', () => {
            window.location.href = '/auth';
        });
    }

    document.getElementById("exit-queue-button").addEventListener('click', () => {
        gameInstance.socket.close();
        window.location.href = '/';
    });
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

function resignGame() {

}
