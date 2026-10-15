function __taiga_complete
    set -l words (commandline -opc)
    set -l cur (commandline -ct)
    set -l verbs new move done rename assign due sprint subtask team client block modify delete search users burndown

    if test (count $words) -eq 1
        taiga complete projects 2>/dev/null
        printf '%s\n' login projects --help --version
        return
    end

    set -l project $words[2]
    contains -- $project (taiga complete projects 2>/dev/null); or return

    set -l id
    set -l verb
    for w in $words[3..-1]
        if test -z "$id"; and string match -qr '^[0-9]+$' -- $w
            set id $w
        else if test -z "$verb"; and contains -- $w $verbs
            set verb $w
        end
    end

    set -l tasks
    test -z "$id"; and set tasks (taiga complete tasks $project 2>/dev/null | string replace -r '^(\d+):(.*)' '$1\t$2')

    switch "$verb"
        case ''
            printf '%s\n' $verbs
            printf '%s\n' $tasks
        case move
            printf '%s\n' $tasks
            taiga complete statuses $project 2>/dev/null
        case assign
            printf '%s\n' $tasks --remove
            taiga complete members $project 2>/dev/null
        case sprint
            printf '%s\n' $tasks --remove
            taiga complete sprints $project 2>/dev/null
        case due team client block
            printf '%s\n' $tasks --remove
        case done delete rename
            printf '%s\n' $tasks
        case subtask
            printf '%s\n' $tasks new list describe delete
        case burndown
            taiga complete sprints $project 2>/dev/null
        case new modify search
            switch $cur
                case 'status:*' '-status:*'
                    taiga complete statuses $project 2>/dev/null | string replace -r '^' (string split -m1 : $cur)[1]:
                case 'sprint:*' '-sprint:*'
                    taiga complete sprints $project 2>/dev/null | string replace -r '^' (string split -m1 : $cur)[1]:
                case '-@*'
                    taiga complete members $project 2>/dev/null | string replace -r '^' -- '-@'
                case '@*'
                    taiga complete members $project 2>/dev/null | string replace -r '^' @
                case '*'
                    printf '%s\n' status: sprint: due: @ +team +client +block
                    test $verb = modify; and printf '%s\n' $tasks -@ -due -sprint -team -client -block
                    test $verb = search; and printf '%s\n' -status: -sprint: -@ -team -client -block
            end
    end
end

complete -c taiga -f -a '(__taiga_complete)'
