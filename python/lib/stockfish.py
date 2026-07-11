# import re
# import time
# import subprocess
#
# class Manager:
#     def __init__(self, stockfish_path: str, timeout: float = 5.0) -> None:
#         self.timeout = timeout
#
#         # Start stockfish engine
#         self.engine = subprocess.Popen(
#             [stockfish_path],
#             stdin=subprocess.PIPE,
#             stdout=subprocess.PIPE,
#             stderr=subprocess.DEVNULL,
#             text=True,
#             bufsize=1
#         )
#
#         # Start engine in uci mode
#         self.engine.stdin.write("uci\n")
#         self.engine.stdin.flush()
#
#         # Wait for uciok with timeout
#         start_time = time.time()
#         while time.time() - start_time < self.timeout:
#             line = self.engine.stdout.readline()
#             if not line:
#                 break
#             if "uciok" in line:
#                 print("✅ Stockfish initialized")
#                 return
#
#         # If we get here, initialization failed
#         self.engine.terminate()
#         raise TimeoutError("Stockfish initialization timeout")
#
#     def eval_fen(self, fen: str) -> str | None:
#         self.engine.stdin.write(f"position fen {fen}\n")
#         self.engine.stdin.flush()
#
#         self.engine.stdin.write("eval\n")
#         self.engine.stdin.flush()
#
#
#         start_time = time.time()
#         while time.time() - start_time < self.timeout:
#             line = self.engine.stdout.readline()
#
#             if not line:
#                 break
#
#             if "NNUE evaluation" in line:
#                 match = re.search(r"[+-]\d+\.\d+", line)
#                 if match:
#                     return float(match.group(0))
#
#             if "Final evaluation" in line:
#                 match = re.search(r"[+-]\d+\.\d+", line)
#                 if match:
#                     return float(match.group(0))
#
#         raise None
#
#
#     def close(self):
#         self.engine.terminate()
#
#
#
#


# import chess
# import chess.engine
#
# class Manager:
#     def __init__(self, stockfish_path: str, timeout: float = 5.0):
#         self.timeout = timeout
#         self.engine = chess.engine.SimpleEngine.popen_uci(stockfish_path)
#         print("✅ Stockfish initialized", flush=True)
#
#     def eval_fen(self, fen: str) -> float:
#         """Get evaluation from White's perspective."""
#         board = chess.Board(fen)
#         info = self.engine.analyse(board, chess.engine.Limit(depth=1))
#         score = info['score']
#
#         if score.is_mate():
#             mate_moves = score.mate()
#             return 100.0 if mate_moves > 0 else -100.0
#
#         # Convert to pawns from White's perspective
#         return score.relative.score() / 100.0
#
#     def close(self):
#         if self.engine:
#             self.engine.quit()
#             self.engine = None
#             print("✅ Stockfish closed", flush=True)
#
#     def __enter__(self):
#         return self
#
#     def __exit__(self, exc_type, exc_val, exc_tb):
#         self.close()
#
#



import re
import time
import subprocess
from typing import Optional
import sys

class Manager:
    def __init__(self, stockfish_path: str, timeout: float = 5.0) -> None:
        self.timeout = timeout
        self.engine = None
        self.buffer = ""  # Store partial lines

        # Start stockfish engine with line buffering
        self.engine = subprocess.Popen(
            [stockfish_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            bufsize=1  # Line buffered
        )

        # Start engine in uci mode
        self.engine.stdin.write("uci\n")
        self.engine.stdin.flush()

        # Wait for uciok with timeout
        start_time = time.time()
        while time.time() - start_time < self.timeout:
            line = self.engine.stdout.readline()
            if not line:
                break
            if "uciok" in line:
                print("✅ Stockfish initialized", flush=True)
                return
        
        # If we get here, initialization failed
        self.engine.terminate()
        raise TimeoutError("Stockfish initialization timeout")

    def eval_fen(self, fen: str) -> Optional[float]:
        """
        Evaluate a FEN position using Stockfish.
        Handles sleeping/resuming gracefully.
        """
        if not self.engine:
            raise RuntimeError("Engine not initialized")

        # Set position
        self.engine.stdin.write(f"position fen {fen}\n")
        self.engine.stdin.flush()

        # Use go depth for reliable results
        self.engine.stdin.write("eval\n")
        self.engine.stdin.flush()

        # Read with timeout - this will "sleep" but resume
        start_time = time.time()
        
        while time.time() - start_time < self.timeout:
            line = self._read_line_with_timeout(0.5)  # Check every 0.5s

            if line is None:
                continue
            
            if not line:
                break
            
            if line.startswith("NNUE evalutation") or line.startswith("Final evaluation"):

                eval = re.search(r"[+-]\d+.\d+", line)
                if eval:
                    score = float(eval.group(0))
                    if "black side" in line:
                        score = -score
                    return score

            else:
                continue

        return None

    def _read_line_with_timeout(self, timeout: float) -> Optional[str]:
        """
        Read a line from stdout with timeout.
        This is what allows the process to "sleep" and "resume".
        """
        import select
        
        # Check if data is available
        ready, _, _ = select.select([self.engine.stdout], [], [], timeout)
        
        if ready:
            return self.engine.stdout.readline()
        return None  # No data available (sleeping)

    def _get_nnue_evaluation(self, fen: str) -> Optional[float]:
        """Fallback: Get NNUE evaluation."""
        if not self.engine:
            return None

        self.engine.stdin.write(f"position fen {fen}\n")
        self.engine.stdin.flush()
        
        self.engine.stdin.write("eval\n")
        self.engine.stdin.flush()
        
        start_time = time.time()
        while time.time() - start_time < self.timeout:
            line = self._read_line_with_timeout(0.5)
            if line is None:
                continue
            if not line:
                break
            
            if "NNUE evaluation" in line:
                match = re.search(r"[+-]\d+\.\d+", line)
                if match:
                    score = float(match.group(0))
                    if "black side" in line:
                        score = -score
                    return score
            
            if "Final evaluation" in line:
                match = re.search(r"[+-]\d+\.\d+", line)
                if match:
                    return float(match.group(0))
        
        return None

    def close(self):
        """Close the Stockfish engine properly."""
        if self.engine:
            try:
                self.engine.stdin.write("quit\n")
                self.engine.stdin.flush()
                time.sleep(0.1)
            except (BrokenPipeError, OSError):
                pass
            self.engine.terminate()
            self.engine = None
            print("✅ Stockfish closed", flush=True)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
