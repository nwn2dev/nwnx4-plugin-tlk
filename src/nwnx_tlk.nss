
// Loads a TLK resolver using two TLK paths (either absolute paths or relative to the NWN2 install path)
//
// The sBaseTlkPath, sUserTlkPath variables can contain tokens, that will be replaced with useful paths:
// - ${NWNX}     Path to the NWNX4 install dir
// - ${NWN2INST} Path to the NWN2 install dir
// - ${NWN2HOME} Path to the home folder, i.e. "Documents/Neverwinter Nights 2"
//
// sResolverName: Custom name for the resolver
// sBaseTlkPath: Path to the TLK file that will be used for strrefs lower than 16_777_216
// sUserTlkPath: Path to the TLK file that will be used for strrefs higher than 16_777_216. Set to "" if you don't have a user TLK file
// -> returns TRUE if all TLKs have been correctly loaded
int TLKLoadResolver(string sResolverName, string sBaseTlkPath, string sUserTlkPath=""){
	string sParam1 = sResolverName + "\n" + sBaseTlkPath;
	if(sUserTlkPath != "")
		sParam1 += "\n" + sUserTlkPath;
	return NWNXGetInt("tlk-rs", "load", sParam1, 0);
}

// Returns TRUE if there is a TLK resolver with the name sResolverName
//
// sResolverName: Name of the resolver
// -> returns TRUE if the resolver has been loaded
int TLKIsResolverLoaded(string sResolverName){
	return NWNXGetInt("tlk-rs", "is_loaded", sResolverName, 0);
}

// Remove the TLK resolver and free memory
//
// sResolverName: Name of the resolver
// -> returns TRUE if the resolver has been removed
int TLKUnoadResolver(string sResolverName){
	return NWNXGetInt("tlk-rs", "unload", sResolverName, 0);
}

// Returns the language ID for a TLK file
//
// sResolverName: Name of the resolver
// nTlk: index of the TLK file in the resolver. 0 for base TLK, 1 for user TLK
// -> returns the language ID of the TLK file (see LANGUAGE_* constants).
//    -1 if there is any error.
int TLKGetLanguageID(string sResolverName, int nTlk){
	return NWNXGetInt("tlk-rs", "get_lang", sResolverName, nTlk);
}

// If set, the TLK entry contains some text
const int TLK_FLAG_TEXT_PRESENT = 1;
// If set, the TLK entry is associated with a sound file
const int TLK_FLAG_SND_PRESENT = 2;
// If set, the sound duration must be read from the sound file
const int TLK_FLAG_SNDLENGTH_PRESENT = 4;

// Returns the string for a given StrRef
//
// sResolverName: Name of the resolver
// nStrref: string ref
// -> returns the string ref flags. See TLK_FLAG_* constants
int TLKGetStrRefFlags(string sResolverName, int nStrref){
	return NWNXGetInt("tlk-rs", "get_flags", sResolverName, nStrref);
}

// Returns the string for a given StrRef
//
// sResolverName: Name of the resolver
// nStrref: string ref
// -> returns the sound duration in seconds
float TLKGetStrRefSoundDuration(string sResolverName, int nStrref){
	return NWNXGetFloat("tlk-rs", "get_sound_length", sResolverName, nStrref);
}

// Returns the string for a given StrRef
//
// sResolverName: Name of the resolver
// nStrref: string ref
// -> returns the sound resource name. "" if there is no sound.
string TLKGetStringByStrRef(string sResolverName, int nStrref){
	return NWNXGetString("tlk-rs", "get", sResolverName, nStrref);
}

// Returns the sound file name associated with a given StrRef
//
// sResolverName: Name of the resolver
// nStrref: string ref
// -> returns the sound resource name. "" if there is no sound.
string TLKGetStrRefSound(string sResolverName, int nStrref){
	return NWNXGetString("tlk-rs", "get_sound_resref", sResolverName, nStrref);
}

