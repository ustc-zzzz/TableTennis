@ECHO OFF

if not exist dist mkdir dist

cargo build --target=i686-pc-windows-msvc

copy %~dp0\src\table_tennis.json %~dp0\dist\com.github.ustc_zzzz.table_tennis.json
copy %~dp0\target\i686-pc-windows-msvc\debug\table_tennis.dll %~dp0\dist\com.github.ustc_zzzz.table_tennis.dll

