cargo b
rmdir /S/Q ..\_OUT
target\debug\copyasyncstd.exe ..\_IN ..\_OUT %*
FC /B ^
 "..\BLUE NOTE\Cannonball Adderley\Somethin' Else\01_Autumn Leaves.flac" ^
 "..\_OUT\BLUE NOTE\Cannonball Adderley\Somethin' Else\01_Autumn Leaves.flac"
