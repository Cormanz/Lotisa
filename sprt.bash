cutechess-cli \
-engine conf="Lotisa A" \
-engine conf="Lotisa B" \
-each \
    tc=inf/10+0.1 \
    book="./resources/opening-book.bin" \
    bookdepth=4 \
-games 2 -rounds 2500 -repeat 2 -maxmoves 200 \
-sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 \
-concurrency 4 \
-ratinginterval 10