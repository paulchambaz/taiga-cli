_taiga() {
  local line=${COMP_LINE:0:COMP_POINT} verbs="new move done rename assign due sprint subtask team client block modify delete search users burndown"
  local -a words cands
  local project id verb cur lead w

  read -ra words <<<"$line"
  [[ $line == *[[:space:]] ]] && words+=("")
  cur=${words[-1]}
  lead=${cur%"${cur##*[:@]}"}

  if ((${#words[@]} == 2)); then
    cands=($(taiga complete projects 2>/dev/null) login projects --help --version)
  else
    project=${words[1]}
    taiga complete projects 2>/dev/null | grep -qxF -- "$project" || return

    for w in "${words[@]:2:${#words[@]}-3}"; do
      if [[ -z $id && $w =~ ^[0-9]+$ ]]; then
        id=$w
      elif [[ -z $verb && " $verbs " == *" $w "* ]]; then
        verb=$w
      fi
    done

    local tasks statuses members sprints
    tasks() { [[ -z $id ]] && taiga complete tasks "$project" 2>/dev/null | cut -d: -f1; }
    statuses() { taiga complete statuses "$project" 2>/dev/null; }
    members() { taiga complete members "$project" 2>/dev/null; }
    sprints() { taiga complete sprints "$project" 2>/dev/null; }

    case $verb in
      "") cands=($verbs $(tasks)) ;;
      move) cands=($(tasks) $(statuses)) ;;
      assign) cands=($(tasks) $(members) --remove) ;;
      sprint) cands=($(tasks) $(sprints) --remove) ;;
      due | team | client | block) cands=($(tasks) --remove) ;;
      done | delete | rename) cands=($(tasks)) ;;
      subtask) cands=($(tasks) new list describe delete) ;;
      burndown) cands=($(sprints)) ;;
      new | modify | search)
        case $cur in
          status:* | -status:*) cands=($(statuses | sed "s/^/${cur%%:*}:/")) ;;
          sprint:* | -sprint:*) cands=($(sprints | sed "s/^/${cur%%:*}:/")) ;;
          -@*) cands=($(members | sed 's/^/-@/')) ;;
          @*) cands=($(members | sed 's/^/@/')) ;;
          *)
            cands=(status: sprint: due: @ +team +client +block)
            [[ $verb == modify ]] && cands+=($(tasks) -@ -due -sprint -team -client -block)
            [[ $verb == search ]] && cands+=(-status: -sprint: -@ -team -client -block)
            ;;
        esac
        ;;
    esac
  fi

  COMPREPLY=($(compgen -W "${cands[*]}" -- "$cur"))
  COMPREPLY=("${COMPREPLY[@]#"$lead"}")
  [[ ${COMPREPLY[0]} == *[:@] ]] && compopt -o nospace 2>/dev/null
}

complete -F _taiga taiga
