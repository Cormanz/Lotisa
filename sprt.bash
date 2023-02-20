rm ./outs/* && /home/corman/cutechess-cli/cutechess-cli \
-engine cmd="target/versions/lotisa" name="Lotisa Aspiration Windows" proto=uci \
-engine cmd="target/versions/lotisa-before" name="Lotisa" proto=uci \
-each \
    tc=inf \
    book="./resources/Titans.bin" \
    bookdepth=4 \
-games 2 -rounds 2500 -repeat 2 \
-sprt elo0=0 elo1=20 alpha=0.05 beta=0.05 -maxmoves 200 \
-ratinginterval 10 \
-pgnout ./outs/games.pgn \
-debug \
-concurrency 10 \
> outs/log.txt