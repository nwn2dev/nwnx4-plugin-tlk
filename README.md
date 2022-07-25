# NWNX4 TLK plugin

Load TLK files by file path, and resolve localized strings

```cpp
// Load a TLK file for resolving strrefs lower than 16_777_216
TLKLoadResolver("german", "${NWN2HOME}/tlk/german/dialog.tlk");

// Load a set of TLK files for resolving any strref
TLKLoadResolver("french",
	"${NWN2HOME}/tlk/french/dialog.tlk",  // Base TLK
	"${NWN2HOME}/tlk/french/dialogF.tlk", // Base female TLK (can be "" for none)
	"${NWN2HOME}/tlk/french/custom.tlk"   // Custom tlk for strrefs higher than 16_777_216
);


// Similar to GetStringByStrRef
string sText = TLKGetStringByStrRef("french", 1337);

// Similar to GetStrRefSound
string sSoundFile = TLKGetStrRefSound("french", 1337);

// Similar to GetStrRefSoundDuration
float fSoundDur = TLKGetStrRefSoundDuration("french", 1337);
```

# Installing

1. Go to the [release page](https://github.com/nwn2dev/nwnx4-plugin-tlk/releases) and download the latest `nwnx4-plugin-tlk.zip`
2. Extract files inside the plugins folder into the nwnx4 plugins folder
3. Extract files inside the nwscript folder into the your module folder
