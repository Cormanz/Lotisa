cutechess-cli \
-engine cmd="target/release/lotisa" arg=A proto=uci \
-engine cmd="target/release/lotisa" arg=B proto=uci \
-each \
    tc=inf \
    book="./resources/Titans.bin" \
    bookdepth=4 \
-games 2 -rounds 2500 -repeat 2 -maxmoves 100 \
-sprt elo0=0 elo1=20 alpha=0.05 beta=0.05 \
-concurrency 10 \
-ratinginterval 10