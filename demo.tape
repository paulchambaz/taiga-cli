Output demo.gif

Set FontSize 20
Set Width 1200
Set Height 600
# Set FontFamily "SauceCodePro Nerd Font Mono"
Set Theme "Gruvbox Dark"
Set Padding 20
Set Shell zsh

Hide
Type "clear"
Enter
Sleep 1s
Show

Sleep 1s

Type "taiga login # use this to login to taiga"
Sleep 1s
Ctrl+C
Sleep 1s

Type "taiga projects"
Sleep 500ms
Enter
Sleep 1s
Sleep 2s

Type "taiga demo --help"
Sleep 500ms
Enter
Sleep 5s
Ctrl+L
Sleep 500ms

Type "# what are the tasks on this project"
Sleep 500ms
Ctrl+C
Type "taiga demo"
Sleep 500ms
Enter
Sleep 2s

Type "# let's create a task"
Sleep 500ms
Ctrl+C
Type "taiga demo new demo task"
Sleep 500ms
Enter
Sleep 2s

Type "taiga demo"
Sleep 500ms
Enter
Sleep 2s

Type "# we can see our new task"
Sleep 500ms
Ctrl+C
Sleep 2s
Ctrl+L
Sleep 500ms

Type "# let's apply some modifications on the task"
Sleep 500ms
Ctrl+C
Sleep 500ms

Type "taiga demo move 4 ready"
Sleep 500ms
Enter
Sleep 2s

Type "taiga demo assign 4 me"
Sleep 500ms
Enter
Sleep 2s

Type "taiga demo 4 due eow"
Sleep 500ms
Enter
Sleep 2s

Type "# here we have moved the task to ready, assigned it to us and set a due date to the end of this week"
Sleep 500ms
Ctrl+C
Sleep 500ms

Type "# let's do all of this and more in a single command"
Sleep 500ms
Ctrl+C
Sleep 500ms

Type "taiga demo mod 4 @me due:eoww giving it a new title stat:in-progress +team -client +block"
Sleep 500ms
Enter
Sleep 3s

Type "taiga demo"
Sleep 500ms
Enter
Sleep 2s

Type "# we can see our task in our list of tasks"
Sleep 500ms
Ctrl+C
Sleep 2s
Ctrl+L
Sleep 500ms

Type "# now let's search the tasks by applying filters on them"
Sleep 500ms
Ctrl+C
Sleep 500ms

Type "# for example, let's select only the task we should finish by the end of the week for the team"
Sleep 500ms
Ctrl+C
Sleep 500ms

Type "taiga demo @me due:eow +team"
Sleep 500ms
Enter
Sleep 2s

Type "# great, and when we're done with that task"
Sleep 500ms
Ctrl+C
Sleep 500ms

Type "taiga demo 1 done"
Sleep 500ms
Enter
Sleep 2s

Type "taiga demo"
Sleep 500ms
Enter
Sleep 5s
