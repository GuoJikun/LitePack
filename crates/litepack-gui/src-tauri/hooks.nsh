; LitePack NSIS 钩子：安装时注册 .zip/.7z 右键菜单动词，卸载时清除。
; 注册路径与动词集合必须与 crates/litepack-gui/src-tauri/src/context_menu.rs 保持一致：
; - 位置：SystemFileAssociations\<ext> 与 <ext> 自身
; - 动词：LitePackOpen / LitePackExtractHere / LitePackExtractTo / LitePackExtractNamed

!macro REGISTER_VERB ext verb display flag
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${ext}\shell\${verb}" "" "${display}"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${ext}\shell\${verb}" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\${ext}\shell\${verb}\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" ${flag} "%1"'
  WriteRegStr HKCU "Software\Classes\${ext}\shell\${verb}" "" "${display}"
  WriteRegStr HKCU "Software\Classes\${ext}\shell\${verb}" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "Software\Classes\${ext}\shell\${verb}\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" ${flag} "%1"'
!macroend

!macro DELETE_VERB ext verb
  DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\${ext}\shell\${verb}"
  DeleteRegKey HKCU "Software\Classes\${ext}\shell\${verb}"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro REGISTER_VERB ".zip" "LitePackOpen" "用 LitePack 打开(&O)" "--open"
  !insertmacro REGISTER_VERB ".zip" "LitePackExtractHere" "直接解压(&X)" "--extract-here"
  !insertmacro REGISTER_VERB ".zip" "LitePackExtractTo" "解压到...(&E)..." "--extract-to"
  !insertmacro REGISTER_VERB ".zip" "LitePackExtractNamed" "解压到同名目录(&N)" "--extract-named"
  !insertmacro REGISTER_VERB ".7z" "LitePackOpen" "用 LitePack 打开(&O)" "--open"
  !insertmacro REGISTER_VERB ".7z" "LitePackExtractHere" "直接解压(&X)" "--extract-here"
  !insertmacro REGISTER_VERB ".7z" "LitePackExtractTo" "解压到...(&E)..." "--extract-to"
  !insertmacro REGISTER_VERB ".7z" "LitePackExtractNamed" "解压到同名目录(&N)" "--extract-named"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro DELETE_VERB ".zip" "LitePackOpen"
  !insertmacro DELETE_VERB ".zip" "LitePackExtractHere"
  !insertmacro DELETE_VERB ".zip" "LitePackExtractTo"
  !insertmacro DELETE_VERB ".zip" "LitePackExtractNamed"
  !insertmacro DELETE_VERB ".7z" "LitePackOpen"
  !insertmacro DELETE_VERB ".7z" "LitePackExtractHere"
  !insertmacro DELETE_VERB ".7z" "LitePackExtractTo"
  !insertmacro DELETE_VERB ".7z" "LitePackExtractNamed"
!macroend
