TAIGA(1) "github.com/paulchambaz/taiga-cli" "General Commands Manual"

# NAME

*taiga* - CLI tool for taiga.io inspired by taskwarrior

# SYNOPSIS

*taiga* *login* [*--address* _<URL>_]++
*taiga* *projects*++
*taiga*++
*taiga* _<PROJECT>_ [_<FILTER>_...]++
*taiga* _<PROJECT>_ *new* _<NAME>_ [_<MODIFIERS>_...]++
*taiga* _<PROJECT>_ _<ID>_ _<COMMAND>_ [_<ARGS>_]++
*taiga* _<PROJECT>_ _<COMMAND>_ _<ID>_ [_<ARGS>_]++
*taiga* _<PROJECT>_ *burndown* [_<SPRINT>_]++
*taiga* _<PROJECT>_ *users*

# DESCRIPTION

*taiga* is a lightweight, terminal-based client for *taiga.io*, designed to
manage the user stories of a kanban quickly, without leaving the shell. It is
inspired by *taskwarrior*: a command is a short sentence, filters select tasks
and modifiers change them. Large operations such as creating projects, adding
members or creating kanban statuses are left to the web interface.

In *taiga*, a task is a user story of the kanban. The tasks nested under a user
story are called subtasks.

Running *taiga* _<PROJECT>_ prints the tasks of a project as a table. The
number in the first column is the number you give to the other commands, it is
the position of the task in the last list that was printed for that project.
Print a list again and the numbers follow the new list. Tasks that are in a
closed status are hidden, unless you search for a status explicitly.

Everything *taiga* learns about your projects (statuses, members, sprints and
the last list of tasks) is kept in a local cache, so most commands need a
single request to Taiga. The cache is refreshed by itself when a name is not
found in it, and by *taiga projects*.

# OPTIONS

*--help*
	Print the help of the command and exit. It can be given after any
	command, for example *taiga demo move 3 --help*.

*--version*
	Print the version and exit.

# GETTING STARTED

Login once, this asks for your username and password and stores the session
in the cache:

```
taiga login
```

If you use a self-hosted instance, give the address of its API:

```
taiga login --address https://taiga.example.org/api/v1
```

Without *--address*, *https://api.taiga.io/api/v1* is used.

Running *taiga* without arguments lists the projects you belong to. Each
project is then a command of its own:

```
taiga
taiga demo
taiga demo new write the manual
```

*taiga projects* refreshes the list of projects from Taiga and prints it. Use
it when you have been added to a new project.

# COMMANDS

Commands that act on one task take the number of the task, either before or
after the command: *taiga demo done 3* and *taiga demo 3 done* are the same.

*taiga* _<PROJECT>_ [_<FILTER>_...]
	Search the tasks of the project. This is the default command, so a
	project alone lists its tasks, and anything that is not another command
	is a filter. *search* can be written out: *taiga demo search +team*.
	See *FILTERS*.

*taiga* _<PROJECT>_ *new* _<NAME>_ [_<MODIFIERS>_...]++
*taiga* _<PROJECT>_ *add* _<NAME>_ [_<MODIFIERS>_...]
	Create a task. The words that are not modifiers are the name. The task
	is created in a single request, in the first status of the project
	unless *status:* says otherwise. See *MODIFIERS*.

*taiga* _<PROJECT>_ *move* _<ID>_ _<STATUS>_
	Move the task to a status of the project.

*taiga* _<PROJECT>_ *done* _<ID>_
	Move the task to the closed status of the project.

*taiga* _<PROJECT>_ *delete* _<ID>_++
*taiga* _<PROJECT>_ *del* _<ID>_
	Delete the task.

*taiga* _<PROJECT>_ *rename* _<ID>_ _<NAME>_...
	Give the task a new name.

*taiga* _<PROJECT>_ *assign* _<ID>_ _<USERNAME>_ [*--remove*]
	Add a user to the task. With *--remove*, take the user off the task
	instead. Adding a user who is already assigned, or removing one who
	is not, is an error. The username *me* is yourself.

*taiga* _<PROJECT>_ *due* _<ID>_ _<DATE>_++
*taiga* _<PROJECT>_ *due* _<ID>_ *--remove*
	Set the due date of the task, or clear it with *--remove*. See *DATES*.

*taiga* _<PROJECT>_ *sprint* _<ID>_ _<SPRINT>_++
*taiga* _<PROJECT>_ *sprint* _<ID>_ *--remove*
	Put the task in a sprint, given by its name or its slug, or take it out
	of its sprint with *--remove*.

*taiga* _<PROJECT>_ *team* _<ID>_ [*--remove*]++
*taiga* _<PROJECT>_ *client* _<ID>_ [*--remove*]++
*taiga* _<PROJECT>_ *block* _<ID>_ [*--remove*]
	Set the team requirement, the client requirement or the block of the
	task. With *--remove*, unset it.

*taiga* _<PROJECT>_ *modify* _<ID>_ _<MODIFIERS>_...++
*taiga* _<PROJECT>_ *mod* _<ID>_ _<MODIFIERS>_...
	Change several fields of the task in one request, only the fields you
	name are touched. See *MODIFIERS*.

*taiga* _<PROJECT>_ *subtask* _<ID>_ _<ACTION>_...
	Manage the tasks nested under a user story. See *SUBTASKS*.

*taiga* _<PROJECT>_ *burndown* [_<SPRINT>_]
	Show the burndown of a sprint. See *BURNDOWN*.

*taiga* _<PROJECT>_ *users*
	List the usernames of the members of the project, as they are used by
	*@* and *assign*.

# MODIFIERS

*new* and *modify* read their arguments the same way. Every argument that looks
like a modifier is one, in any order, and what remains is the name of the task.
The name must be contiguous: *new fix @paul the bug* is refused, because the
modifier *@paul* splits the name in two. Words before or after the modifiers
are fine.

*status:*_<STATUS>_, *stat:*_<STATUS>_
	The status of the task, given by its slug (for example *in-progress*).
	Without it, a new task is created in the first status of the project.

*@*_<USERNAME>_
	Assign the task to a user. It can be given several times to assign
	several users. In *modify*, the user is added to the users already on
	the task.

*-@*_<USERNAME>_
	Take a user off the task. Only in *modify*.

*due:*_<DATE>_
	The due date of the task. See *DATES*.

*-due*
	Remove the due date of the task. Only in *modify*.

*sprint:*_<SPRINT>_
	Put the task in a sprint, given by its name or its slug.

*-sprint*
	Take the task out of its sprint. Only in *modify*.

*+team*, *-team*++
*+client*, *-client*++
*+block*, *-block*
	Set or unset the team requirement, the client requirement and the block.

Modifiers that take a single value (*status:*, *due:*, *sprint:*) can be given
once. Giving *+team* and *-team* together, or *due:* with *-due*, or *sprint:*
with *-sprint*, is an error and prints the help of the command. An empty value,
such as *status:*, is also an error. Anything else that starts with *+* or *-*,
or contains a colon, is refused rather than silently becoming part of the name:
if you want a colon in a name, use *rename*.

# FILTERS

Any argument after the project that is not another command is a filter. Filters
are combined: a task is listed only if it satisfies all of them.

*status:*_<STATUS>_, *stat:*_<STATUS>_
	The task is in the status. Given several times, the task may be in any
	of them.

*-status:*_<STATUS>_, *-stat:*_<STATUS>_
	The task is not in the status.

*@*_<USERNAME>_
	The task is assigned to the user. Given several times, the task may be
	assigned to any of them. *@me* is yourself.

*-@*_<USERNAME>_
	The task is not assigned to the user.

*sprint:*_<SPRINT>_, *-sprint:*_<SPRINT>_
	The task is in, or is not in, the sprint.

*due:*_<DATE>_
	The task is due on that date or before. See *DATES*. Tasks without a due
	date are not listed.

*due:*
	With nothing after the colon: the task has no due date.

*+team*, *-team*, *+client*, *-client*, *+block*, *-block*
	The task has, or has not, the team requirement, the client requirement
	or the block.

_<WORD>_...
	The remaining words are a fuzzy query matched against the name of the
	task: the letters must appear in order, but not next to each other.
	A word that starts with *-* or *+* but is not a filter is part of the
	query.

Closed tasks are hidden, unless at least one *status:* filter is given.

The table is sorted by status, the furthest along first, then by due date, the
soonest first, with tasks without a due date last. The columns are the number,
the status, the time left before the due date, the name and the usernames of
the assigned users. The columns *SPRINT*, *T* (team), *C* (client) and *B*
(blocked) only appear when at least one task needs them. The time left is
written *3h*, *2d*, *1w*, *4m* or *1y*, with a leading *-* when the date is
past.

# DATES

Dates are read by *temporis*, so you can write them the way you would say them:

[[ *Form*
:[ *Meaning*
|  _2030-05-05_, _05-05-2030_
:  A calendar date
|  _05/05_
:  A day and month, in the current year
|  _today_, _tomorrow_, _yesterday_
:  Relative to today
|  _monday_, _tue_, _nfriday_
:  The coming weekday, or the one of next week
|  _eow_, _eom_, _eoq_, _eoy_
:  The end of the week, month, quarter or year
|  _sow_, _som_, _soq_, _soy_
:  The start of the week, month, quarter or year
|  _eonw_, _eonm_, _eonq_, _eony_
:  The end of the next week, month, quarter or year
|  _5d_, _2w_, _3m_, _1y_
:  In so many days, weeks, months or years

An invalid date prints an error and nothing is sent to Taiga.

# SUBTASKS

A user story can contain smaller tasks. They are managed through the number of
the story:

*taiga* _<PROJECT>_ _<ID>_ *subtask* *new* _<NAME>_... [*@*_<USERNAME>_] [*desc:*_<TEXT>_]
	Create a subtask under the story. The words that are not *@* or
	*desc:* are the name. The description is a single argument, quote it if
	it has spaces.

*taiga* _<PROJECT>_ _<ID>_ *subtask* *list*
	List the subtasks of the story with their status, description and
	assigned users. The numbers of this list are the ones used below, and
	the descriptions are fetched in parallel, so this is the slowest command.

*taiga* _<PROJECT>_ _<ID>_ *subtask* *describe* _<N>_ _<TEXT>_...++
*taiga* _<PROJECT>_ _<ID>_ *subtask* *desc* _<N>_ _<TEXT>_...
	Set the description of the subtask _<N>_ of the last *list*.

*taiga* _<PROJECT>_ _<ID>_ *subtask* *delete* _<N>_++
*taiga* _<PROJECT>_ _<ID>_ *subtask* *del* _<N>_
	Delete the subtask _<N>_ of the last *list*.

# BURNDOWN

*taiga* _<PROJECT>_ *burndown* shows the burndown of the sprint that is running
today. If none is running, it shows the open sprint that ends last, and if every
sprint is closed it says so. Give a sprint, by name or slug, to look at another
one.

The output starts with the name and the dates of the sprint, then the number of
user stories and of tasks completed out of the total, then one line per day with
the points still open and the points an ideal sprint would have left.

```
Sprint 1  2026-10-12 to 2026-10-26
stories 2/5   tasks 4/12

DAY         OPEN  OPTIMAL
2026-10-12  21.0  21.0
2026-10-13  18.0  19.0
```

# SHELL COMPLETION

Completion files for *zsh*, *bash* and *fish* are in the *completions* directory
of the source, and *just install* (as well as the nix package) puts them where
each shell looks for them:

- *zsh*: *share/zsh/site-functions/\_taiga*. The directory must be in your
  *fpath* before *compinit* runs.
- *bash*: *share/bash-completion/completions/taiga*, loaded by *bash-completion*.
- *fish*: *share/fish/vendor_completions.d/taiga.fish*.

They complete the projects, the verbs, the numbers of the tasks (with their
names), the statuses, the sprints, the usernames and the modifiers. The names
come from the cache, so they follow your projects, and no request is made.
They get them from *taiga complete* _<KIND>_ _[PROJECT]_, a hidden command that
prints one name per line, where _<KIND>_ is one of *projects*, *statuses*,
*members*, *sprints* or *tasks*.

# FILES

*~/.cache/taiga/config* (more exactly *$XDG_CACHE_HOME/taiga/config*)
	The cache: your session, the projects with their statuses, members and
	sprints, and the last list of tasks of each project. It is a single
	file, only readable by you, and it also contains the *password* you gave
	to *taiga login*, so that the session can be renewed when it expires.
	Delete it to log out. A cache from another version of *taiga* is
	ignored and you are asked to login again.

# EXAMPLES

Create a task for the team, assigned to yourself, due at the end of the week:

```
taiga demo new fix the login page @me due:eow +team
```

Move the third task to a status, then close it:

```
taiga demo 3 move in-progress
taiga demo 3 done
```

Change several things at once, and rename the task:

```
taiga demo mod 3 @arjun -@me due:2030-05-05 +block better title
```

Remove the due date and take the task out of its sprint:

```
taiga demo mod 3 -due -sprint
```

What is on my plate this week, and what is blocked:

```
taiga demo @me due:eow
taiga demo +block
```

Everything in a sprint that is not done, whatever the user:

```
taiga demo sprint:sprint-1 -status:done
```

The tasks nobody has taken yet, which match "pars":

```
taiga demo status:new pars
```

Break a story down into subtasks, and write what one of them is about:

```
taiga demo 3 subtask new write the tests @paul
taiga demo 3 subtask list
taiga demo 3 subtask describe 1 cover the date parser
```

How is the current sprint doing:

```
taiga demo burndown
```

# EXIT STATUS

*0* on success. *1* on a usage error, which also prints the help of the
command, or when an error occurs, which is printed on the standard error with
its cause.

# AUTHORS

This program was written by Paul Chambaz. It is licensed under the GPLv3, for
more information, please read the *LICENSE* file.
