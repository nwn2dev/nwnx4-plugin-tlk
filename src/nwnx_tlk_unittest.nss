#include "nwnx_tlk"

void Error(string sMsg){
	SendMessageToPC(GetFirstPC(), sMsg);
	WriteTimestampedLogEntry(sMsg);
}

void Assert(int bCond, string sScript, int nLine, string sMessage=""){
	if(!bCond){
		Error("ERROR:" + sScript + ".nss:" + IntToString(nLine) + ": Assert failed" + (sMessage != "" ? ": " + sMessage : ""));
	}
}
void AssertEqS(string sA, string sB, string sScript, int nLine, string sMessage=""){
	if(sA != sB){
		Error("ERROR:" + sScript + ".nss:" + IntToString(nLine) + ": Assert failed: '" + sA + "' != '" + sB + "'" + (sMessage != "" ? ": " + sMessage : ""));
	}
}
void AssertEqI(int nA, int nB, string sScript, int nLine, string sMessage=""){
	if(nA != nB){
		Error("ERROR:" + sScript + ".nss:" + IntToString(nLine) + ": Assert failed: " + IntToString(nA) + " != " + IntToString(nB) + (sMessage != "" ? ": " + sMessage : ""));
	}
}
void AssertEqF(float fA, float fB, float fPrecision, string sScript, int nLine, string sMessage=""){
	if(fabs(fA - fB) > fPrecision){
		Error("ERROR:" + sScript + ".nss:" + IntToString(nLine) + ": Assert failed: " + FloatToString(fA, 0) + " != " + FloatToString(fA, 0) + " (precision: " + FloatToString(fPrecision, 0) + ")" + (sMessage != "" ? ": " + sMessage : ""));
	}
}

void main()
{
	Assert(TLKLoadResolver("stock", "${NWN2HOME}/test/dialog.tlk"), __FILE__, __LINE__);
	Assert(TLKIsResolverLoaded("stock"), __FILE__, __LINE__);
	AssertEqI(TLKGetTalkTableLanguage("stock", 0), LANGUAGE_FRENCH, __FILE__, __LINE__);
	AssertEqI(TLKGetTalkTableLanguage("stock", 1), 0, __FILE__, __LINE__);

	AssertEqI(        TLKGetStrRefFlags("stock", 0), TLK_FLAG_TEXT_PRESENT, __FILE__, __LINE__);
	AssertEqF(TLKGetStrRefSoundDuration("stock", 0), 0.0, 0.0, __FILE__, __LINE__);
	AssertEqS(     TLKGetStringByStrRef("stock", 0), "Bad Strref", __FILE__, __LINE__);
	AssertEqS(        TLKGetStrRefSound("stock", 0), "", __FILE__, __LINE__);

	AssertEqI(        TLKGetStrRefFlags("stock", 1), TLK_FLAG_TEXT_PRESENT, __FILE__, __LINE__);
	AssertEqF(TLKGetStrRefSoundDuration("stock", 1), 0.0, 0.0, __FILE__, __LINE__);
	AssertEqS(     TLKGetStringByStrRef("stock", 1), "Barbares", __FILE__, __LINE__);
	AssertEqS(        TLKGetStrRefSound("stock", 1), "", __FILE__, __LINE__);

	AssertEqI(        TLKGetStrRefFlags("stock", 10), TLK_FLAG_TEXT_PRESENT, __FILE__, __LINE__);
	AssertEqF(TLKGetStrRefSoundDuration("stock", 10), 0.0, 0.0, __FILE__, __LINE__);
	AssertEqS(     TLKGetStringByStrRef("stock", 10), "Moine", __FILE__, __LINE__);
	AssertEqS(        TLKGetStrRefSound("stock", 10), "", __FILE__, __LINE__);

	AssertEqI(        TLKGetStrRefFlags("stock", 198), 0, __FILE__, __LINE__);
	AssertEqF(TLKGetStrRefSoundDuration("stock", 198), 0.0, 0.0, __FILE__, __LINE__);
	AssertEqS(     TLKGetStringByStrRef("stock", 198), "", __FILE__, __LINE__);
	AssertEqS(        TLKGetStrRefSound("stock", 198), "", __FILE__, __LINE__);

	AssertEqI(        TLKGetStrRefFlags("stock", 76347), TLK_FLAG_TEXT_PRESENT | TLK_FLAG_SND_PRESENT | TLK_FLAG_SNDLENGTH_PRESENT, __FILE__, __LINE__);
	AssertEqF(TLKGetStrRefSoundDuration("stock", 76347), 0.0, 0.0, __FILE__, __LINE__);
	AssertEqS(     TLKGetStringByStrRef("stock", 76347), "Faites quelque chose !", __FILE__, __LINE__);
	AssertEqS(        TLKGetStrRefSound("stock", 76347), "vs_nx0xanom_dyin", __FILE__, __LINE__);

	Assert(TLKUnoadResolver("stock"), __FILE__, __LINE__);
	Assert(!TLKIsResolverLoaded("stock"), __FILE__, __LINE__);
	Assert(!TLKUnoadResolver("stock"), __FILE__, __LINE__);
	AssertEqI(        TLKGetStrRefFlags("stock", 0), 0, __FILE__, __LINE__);
	AssertEqF(TLKGetStrRefSoundDuration("stock", 0), 0.0, 0.0, __FILE__, __LINE__);
	AssertEqS(     TLKGetStringByStrRef("stock", 0), "", __FILE__, __LINE__);
	AssertEqS(        TLKGetStrRefSound("stock", 0), "", __FILE__, __LINE__);
}