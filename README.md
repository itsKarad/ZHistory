# ZHistory

## Features
1. Interactive filtering for efficient navigation through history => Aditya 


No keywords -> last 50 commands.
Keywords -> Search
Multiple keywords -> Tokenise, search

When results are shown, take input from user to filter to a particular keyword. Reduce results accordingly.
When user clicks ENTER, execute the corresponding command.
Numbering all commands.

2. Frequency analysis metrics for command usage patterns => Hitarth

50 commands. First few keywords check -> Analytics (hashmap).
map<string, frequency> mp;

On the fly

Most common commands per day, per week, per month.

3. Compatibility with both zsh and bash shells.

Test with bash. 
Test with ZSH.

4. Command history analysis by hour, day, and week. => Hitarth
zhistory --hr
zhistory --week
zhistory --month

Go to results page and allow interactive searching again.

