; LitePack NSIS 钩子：安装时注册 .zip/.7z 右键菜单「直接解压」，卸载时清除。

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "Software\Classes\.zip\shell\LitePackExtractHere" "" "直接解压(&X)"
  WriteRegStr HKCU "Software\Classes\.zip\shell\LitePackExtractHere" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\.zip\shell\LitePackExtractHere\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" --extract-here "%1"'
  WriteRegStr HKCU "Software\Classes\.7z\shell\LitePackExtractHere" "" "直接解压(&X)"
  WriteRegStr HKCU "Software\Classes\.7z\shell\LitePackExtractHere" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\.7z\shell\LitePackExtractHere\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" --extract-here "%1"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegKey HKCU "Software\Classes\.zip\shell\LitePackExtractHere"
  DeleteRegKey HKCU "Software\Classes\.7z\shell\LitePackExtractHere"
!macroend
