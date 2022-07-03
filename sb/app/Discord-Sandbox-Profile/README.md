# Discord Sandbox Profile for Apple's Sandbox

## NOTE: DEFUNCT SINCE 2019
**NOTE:**  
This guide assumes you're running Discord from $HOME/Applications/Discord.app rather than /Applications/Discord.app  
To change this edit this line:  

````
(allow process-exec* 
    (regex #"^/Users/[a-zA-Z]+/Applications/Discord.app/")
)
````

This guide further assumes your user-ID is 503. If it's not. Find out what it is by issuing `id` and change lines containing 503 accordingly.

## How to start Discord using this profile.

To start Discord using this profile put it in a location of your choice and change this line in the profile accordingly:

````
(regex #"^/Users/[a-zA-Z]+/\.custom-sandbox-profiles/.*")
````

Then start Discord with the profile using `sandbox-exec -f [path-to-profile] path-to-Discord.app/Contents/MacOS/Discord`

Debugging the Profile can be done via  

````
sudo /Applications/Utilities/Console.app/Contents/MacOS/Console
````

Just filter for sandbox and/or discord.

To allow trace every-permission Discord asks for and write it out into a file uncomment this line.  

````
;;(trace "discord-trace.sb")
````



## Wanna start Discord using a one-click method?

Just use this AppleScript

````javascript
var app = Application.currentApplication();
	app.includeStandardAdditions = true;

//Assuming you export the script as .app-Bundle and put the profile into Contents/Resources/
var discordProfile = app.pathToResource("discord.sb");

app.doShellScript('sandbox-exec -f '+discordProfile+' $HOME/Applications/Discord.app/Contents/MacOS/Discord &>/dev/null &');
````

To change the Script's .app-Bundle's Icon to Discord's copy Discords Icon from within Discord.app/Contents/Resources into the same folder in your script's app-Bundle. And change  this

````xml
<key>CFBundleIconFile</key>
	<string>applet</string>
````

in your script's .app/Contents/Info.plist accordingly.
