docker run --rm -v /home/jake/Documents/school-projects/multiplayer_platformer/docs:/pandoc dalibo/pandocker -V classoption=oneside \
    --toc --number-sections --top-level-division=chapter --template=eisvogel \
    --listings -o ifttt-system-design-v7.pdf \
    docs/ifttt/title.txt \
    docs/ifttt/ifttt.md