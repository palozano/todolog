" todolog.vim - Vim quickfix integration for todolog
" Maintainer: Pablo Lozano
" License: MIT

if exists('g:loaded_todolog_vim')
  finish
endif
let g:loaded_todolog_vim = 1

if !exists('g:todolog_command')
  let g:todolog_command = 'todolog'
endif

function! s:command(args) abort
  return shellescape(g:todolog_command) . ' ' . join(map(copy(a:args), 'shellescape(v:val)'), ' ')
endfunction

function! s:scan(...) abort
  let l:root = a:0 ? a:1 : getcwd()
  execute 'silent !' . s:command(['scan', l:root])
  redraw!
endfunction

function! s:tasks(...) abort
  let l:bang = a:0 && a:1 ==# '!'
  let l:output = system(s:command(['list', '--open', '--quickfix']))
  if v:shell_error
    echoerr l:output
    return
  endif

  cexpr l:output
  if l:bang
    botright copen
  else
    copen
  endif
endfunction

function! s:set_status(command, id) abort
  let l:output = system(s:command([a:command, a:id]))
  if v:shell_error
    echoerr l:output
    return
  endif
  call s:tasks()
endfunction

command! -bar -nargs=? -complete=dir TodologScan call s:scan(<f-args>)
command! -bar -bang TodologTasks call s:tasks(<q-bang>)
command! -bar -nargs=1 TodologDone call s:set_status('done', <q-args>)
command! -bar -nargs=1 TodologOpen call s:set_status('open', <q-args>)

nnoremap <silent> <Plug>(todolog-scan) :TodologScan<CR>
nnoremap <silent> <Plug>(todolog-tasks) :TodologTasks<CR>
