target triple = "x86_64-unknown-linux-gnu"
declare noundef i8 @_RNvCs2s9PSdYtRVi_4peer3add(i8 noundef , i8 noundef ) nounwind
define i8 @lustr_import_abi_1c95d373ab78c2ee_4d456a34177def63(i8 %a0, i8 signext noundef %a1) nounwind {
entry:
  %value = call noundef i8 @_RNvCs2s9PSdYtRVi_4peer3add(i8 noundef %a0, i8 noundef %a1)
  ret i8 %value
}
