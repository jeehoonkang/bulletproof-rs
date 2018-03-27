#include <signal.h>
#include <setjmp.h>
#include <stddef.h>
#include <memory.h>

// Calling environment for recovering from segmentation fault.  `__thread` for thread-safe handling
// of sigsegv.
__thread sigjmp_buf jbuf;

// Number of nested bulletproof sections.
__thread size_t count;

// Old handler.
__thread struct sigaction old_handler;

// Longjmp to the stored environment.
static void handle_sigsegv(int sig __attribute__((unused)),
                           siginfo_t *si __attribute__((unused)),
                           void *unused __attribute__((unused))) {
  siglongjmp(jbuf, 1);
}

// Install the sigsegv handler, and save the old handler.
size_t bulletproof_section_begin() {
  if (count++ > 0) { return 0; }

  struct sigaction new_handler;
  new_handler.sa_flags = SA_SIGINFO;
  sigemptyset(&new_handler.sa_mask);
  new_handler.sa_sigaction = handle_sigsegv;

  return sigaction(SIGSEGV, &new_handler, &old_handler);
}

// Uninstall the sigsegv handler, and reinstall the old handler.
//
// # Safety
//
// If there's no open bulletproof sections, you should not call it.
size_t bulletproof_section_end() {
  if (--count > 0) { return 0; }

  return sigaction(SIGSEGV, &old_handler, NULL);
}

// Load `size_t` from `from`, and store it to `to`.
//
// # Safety
//
// You should call it inside a bulletproof section.
//
// # Returns
//
// If `from` is invalid, return 1. Otherwise, return 0.
size_t bulletproof_load(const size_t *from, size_t *to) {
  if (sigsetjmp(jbuf, -1) != 0) {
    return 1;
  }

  *to = *from;
  return 0;
}

// Load `size` bytes from `from`, and store it to `to`.
//
// # Safety
//
// You should call it inside a bulletproof section.
//
// # Returns
//
// If `from` is invalid, return 1. Otherwise, return 0.
size_t bulletproof_load_bytes(const char *from, char *to, size_t size) {
  if (sigsetjmp(jbuf, -1) != 0) {
    return 1;
  }

  memcpy((void *) to, (void *) from, size);
  return 0;
}
