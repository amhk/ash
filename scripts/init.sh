function ash() {
    output=$(<template:binary> $*)
    case $? in
        0)
            echo -e "$output"
            return 0
            ;;
        27)
            eval "$output"
            return $?
            ;;
        *)
            return 1
            ;;
    esac
}

