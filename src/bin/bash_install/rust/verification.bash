( which cargo > /dev/null && which rustc > /dev/null ) || (
    echo "It looks like you do not have rust installed, go do that"
    exit 1
)
