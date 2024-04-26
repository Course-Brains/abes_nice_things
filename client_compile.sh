#! bin/bash
# A template for the client_compile generated script


#First segment(rustup checks)
while true; do
    echo 'Do you have rustup? (y/n)'
    read input
    if [ $input == 'y' ]; then
        echo 'yay'
        break
    elif [ $input == 'n' ]; then
        echo 'On Unix systems(MacOS/Linux):'
        echo 'Run : curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh'
        echo 'On Windows:'
        echo 'Download: 'https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe' then run the file'
        echo 'On success: rerun this file'
        exit
    fi
done
#End first segment

#Second segment(crate/toml/lock generation)
title='Test'
toml=''
lock=''
cargo new '$title'
cd '$title'
echo '$toml' >> 'Cargo.toml'
echo '$lock' > 'Cargo.lock'
cd src
#End second segment

#Third segment(every file)
file=''
data=''
touch "$file"
echo "$data" > "$file"
#End third segment(remember to go back to src at end)

#Fourth segment(last)
cd ../
cargo run
#End fourth segment