set terminal png size 800,600
set output "speed.png"

set title "File Transfer Speed"

set xlabel "Data size (MB)"
set ylabel "Time (s)"

set datafile separator ","
plot "speed.csv" u 1:2 with linespoints title "DFS", "speed.csv" u 1:3 with linespoints title "IPFS"
