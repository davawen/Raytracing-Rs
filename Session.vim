let SessionLoad = 1
let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1
let v:this_session=expand("<sfile>:p")
silent only
silent tabonly
cd /mnt/Projects/Rust/Raytracing
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
let s:shortmess_save = &shortmess
if &shortmess =~ 'A'
  set shortmess=aoOA
else
  set shortmess=aoO
endif
badd +230 src/main.rs
badd +98 src/material.rs
badd +486 ~/.config/nvim/lua/config.lua
badd +233 src/intersection.rs
badd +7 term:///mnt/Projects/Rust/Raytracing//574806:/usr/bin/fish
badd +16 Cargo.toml
argglobal
%argdel
$argadd src/main.rs
tabnew +setlocal\ bufhidden=wipe
tabrewind
edit src/material.rs
argglobal
balt src/intersection.rs
setlocal fdm=expr
setlocal fde=nvim_treesitter#foldexpr()
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=1
setlocal fml=1
setlocal fdn=20
setlocal fen
97
normal! zo
98
normal! zo
108
normal! zo
113
normal! zo
let s:l = 102 - ((28 * winheight(0) + 28) / 56)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 102
normal! 014|
tabnext
argglobal
if bufexists(fnamemodify("term:///mnt/Projects/Rust/Raytracing//574806:/usr/bin/fish", ":p")) | buffer term:///mnt/Projects/Rust/Raytracing//574806:/usr/bin/fish | else | edit term:///mnt/Projects/Rust/Raytracing//574806:/usr/bin/fish | endif
if &buftype ==# 'terminal'
  silent file term:///mnt/Projects/Rust/Raytracing//574806:/usr/bin/fish
endif
balt src/main.rs
setlocal fdm=expr
setlocal fde=nvim_treesitter#foldexpr()
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=1
setlocal fml=1
setlocal fdn=20
setlocal fen
let s:l = 7 - ((6 * winheight(0) + 28) / 57)
if s:l < 1 | let s:l = 1 | endif
keepjumps exe s:l
normal! zt
keepjumps 7
normal! 02|
tabnext 1
if exists('s:wipebuf') && len(win_findbuf(s:wipebuf)) == 0 && getbufvar(s:wipebuf, '&buftype') isnot# 'terminal'
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20
let &shortmess = s:shortmess_save
let s:sx = expand("<sfile>:p:r")."x.vim"
if filereadable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &g:so = s:so_save | let &g:siso = s:siso_save
set hlsearch
nohlsearch
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
