#include <signal.h>
#include <setjmp.h>
#include <stddef.h>
#include <memory.h>

// Calling environment for recovering from segmentation fault.  `__thread` for thread-safe handling
// of sigsegv.
__thread sigjmp_buf jbuf;

// Longjmps to the stored environment.
static void bulletproof_handler(int sig __attribute__((unused)),
                                siginfo_t *si __attribute__((unused)),
                                void *unused __attribute__((unused))) {
  siglongjmp(jbuf, 1);
}

// Installs the SIGSEGV handler.
size_t bulletproof_register() {
  struct sigaction new_handler;
  new_handler.sa_flags = SA_SIGINFO;
  sigemptyset(&new_handler.sa_mask);
  new_handler.sa_sigaction = bulletproof_handler;

  return sigaction(SIGSEGV, &new_handler, NULL);
}

// Loads `size_t` from `loc`, and store it to `dst`.
//
// # Safety
//
// You should call it after calling `bulletproof_register()`.
//
// # Returns
//
// If `loc` is invalid, return 1. Otherwise, return 0.
size_t bulletproof_load(const size_t *loc, size_t *dst) {
  if (sigsetjmp(jbuf, -1) != 0) {
    return 1;
  }

  *dst = *loc;
  return 0;
}

// Stores `val` of type `size_t` into `loc`.
//
// # Safety
//
// You should call it after calling `bulletproof_register()`.
//
// # Returns
//
// If `loc` is invalid, return 1. Otherwise, return 0.
size_t bulletproof_store(size_t *loc, size_t val) {
  if (sigsetjmp(jbuf, -1) != 0) {
    return 1;
  }

  *loc = val;
  return 0;
}

// Loads `size` bytes from `loc`, and store it to `dst`.
//
// # Safety
//
// You should call it after calling `bulletproof_register()`.
//
// `dst` should be a valid buffer with size at least `size`.
//
// # Returns
//
// If `loc` is invalid, return 1. Otherwise, return 0.
size_t bulletproof_load_bytes(const char *loc, char *dst, size_t size) {
  if (sigsetjmp(jbuf, -1) != 0) {
    return 1;
  }

  memcpy((void *) dst, (void *) loc, size);
  return 0;
}

// Stores `val` into `loc`.
//
// # Safety
//
// You should call it after calling `bulletproof_register()`.
//
// `dst` should be a valid buffer with size at least `size`.
//
// # Returns
//
// If `loc` is invalid, return 1. Otherwise, return 0.
size_t bulletproof_store_bytes(char *loc, const char *src, size_t size) {
  if (sigsetjmp(jbuf, -1) != 0) {
    return 1;
  }

  memcpy((void *) loc, (void *) src, size);
  return 0;
}
