#/bin/bash

intervals=(0 20 50 100 1000 5000)
durations=(100 200 300 400 500 1000 1500 2000 5000)

out='results.csv'

printf '%s' 'duration,' > $out
for interval in "${intervals[@]}"
do
    printf '%s' "$interval," >> $out
done

for duration in "${durations[@]}"
do
    printf '\n%s,' "$duration" >> $out
    for interval in "${intervals[@]}"
    do
        ./target/debug/net-timing-channel benchmark $interval $duration >> $out
        printf '%s' ',' >> $out
        sleep 2
        echo finished $duration $interval
    done
done

