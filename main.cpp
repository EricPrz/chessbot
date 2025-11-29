#include <cctype>
#include <chrono>
#include <cstddef>
#include <cstdio>
#include <memory>
#include <regex>
#include <unordered_map>
#include <iostream>
#include <ostream>
#include <stack>
#include <string>
#include <thread>
#include <vector>

bool running = true;

void read_smth(){
    std::cout << "hi, im listening" << std::endl;
    std::string msg;
    while (running){
        std::getline(std::cin, msg);
        std::cout << msg << std::endl;

        if (msg == "q"){
            running = false;
            return;
        }
    }
}

void print_smth(){
    while (running){
        std::this_thread::sleep_for(std::chrono::seconds(2));
        std::cout << "smth" << std::endl;
    }
}

const int SIZE = 8;

enum Color { white, black };
enum PieceType { Rook, Bishop, Knight, Queen, King, Pawn, none };
enum MoveType { Illegal, Move, Capture, Enroque, Promocion };
enum PieceLetter {p, r, b, n, k, q, P, R, B, N, K, Q, _};

struct Piece {
    PieceType piece;
    Color color;

    Piece(): piece(Pawn), color(black) {}
    Piece(PieceType p, Color c): piece(p), color(c) {}
};

struct Position {
    int x;
    int y;

    Position(int x, int y): x(x), y(y) {}
};

struct Board {
    std::vector<std::vector<char>> board;
};

std::unordered_map<char, Piece> piece_to_board = {
    {'p', Piece(Pawn, black)},
    {'b', Piece(Bishop, black)},
    {'r', Piece(Rook, black)},
    {'n', Piece(Knight, black)},
    {'q', Piece(Queen, black)},
    {'k', Piece(King, black)},

    {'_', Piece(none, white)},

    {'P', Piece(Pawn, white)},
    {'B', Piece(Bishop, white)},
    {'R', Piece(Rook, white)},
    {'N', Piece(Knight, white)},
    {'Q', Piece(Queen, white)},
    {'K', Piece(King, white)}
};

Piece piece_at(Position pos, Board board){
    char piece = board.board.at(pos.y).at(pos.x);

    return piece_to_board[piece];
}

std::vector<char> fenn_to_row(std::string fen){
    std::vector<char> row;

    int count = 0;
    auto end = fen.end();
    auto begin = fen.begin();

    while (begin != end){
	char lowerC = std::tolower(*begin);
	if (lowerC <= '9' && lowerC >= '0'){
	    int a = lowerC-'0';
	    for (int i = 0; i < a; i++){
		row.push_back('_');
		std::cout << '_';
	    }
	} else {
	    row.push_back(*begin);
	    std::cout << *begin;
	}
	++count;
	++begin;
    }

    std::cout << std::endl;

    return row;
}

void board_from_fen(std::string fen){
    Board board;

    std::string fenn = "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3";
    std::regex re("[\\w-]+");

    std::sregex_iterator begin = std::sregex_iterator(fenn.begin(), fenn.end(), re);
    std::sregex_iterator end;

    int count = 0;

    // Iterate over all matches and extract the text between slashes
    while (begin != end) {
        // Get the match object
        std::smatch m = *begin;


	if (count < 8){
	    // Decode to board a row	
	    std::vector<char>row = fenn_to_row(m.str());
	    board.board.push_back(row);
	} else {
	    std::cout << m.str() << std::endl;
	}

        // Move to the next match
        ++begin;
	++count;
    }

    std::cout << count << std::endl;

}

class Game {
private:
    Board board;
    Color turn;
    std::stack<std::string> moves;


public:
    Game(){
	board = {{
		{'r', 'b', 'n', 'q', 'k', 'n', 'b', 'r'},
		{'p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'},
		{'_', '_', '_', '_', '_', '_', '_', '_'},
		{'_', '_', '_', '_', '_', '_', '_', '_'},
		{'_', '_', '_', '_', '_', '_', '_', '_'},
		{'_', '_', '_', '_', '_', '_', '_', '_'},
		{'P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'},
		{'R', 'B', 'N', 'Q', 'K', 'N', 'B', 'R'}
	}};

	turn = white;
    }

    Board get_board(){
	return board;
    }
};





int main(){
    Game game;

    board_from_fen("r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3");

    std::thread m(read_smth);
    std::thread t(print_smth);

    t.join();
    m.join();

    return 0;
}
