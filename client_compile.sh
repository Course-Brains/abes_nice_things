#! bin/bash
# A template for the client_compile generated script

while true; do
    echo "Do you have rustup? (y/n)"
    read input
    if [ $input == "y" ]; then
        echo "yay"
        break
    elif [ $input == "n" ]; then
        echo "On Unix systems(MacOS/Linux):"
        echo "Run : 'curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh'"
        echo "On Windows:"
        echo "Download: 'https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe' then run the file"
        echo "On success: rerun file"
        exit
    fi
done

cargo help