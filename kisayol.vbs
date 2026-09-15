Set fso = CreateObject("Scripting.FileSystemObject")
Set ws = CreateObject("WScript.Shell")
Set lnk = ws.CreateShortcut(ws.SpecialFolders("Desktop") & "\Finansal Hesaplaci.lnk")
lnk.TargetPath = fso.GetParentFolderName(WScript.ScriptFullName) & "\start.bat"
lnk.WorkingDirectory = fso.GetParentFolderName(WScript.ScriptFullName)
lnk.Description = "Finansal Hesaplaci"
lnk.Save
WScript.Echo "Tamam!"
