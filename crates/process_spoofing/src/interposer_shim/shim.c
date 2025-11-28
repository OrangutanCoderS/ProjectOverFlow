#define _GNU_SOURCE
#include <dlfcn.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <sys/un.h>
#include <sys/socket.h>
#include <stdio.h>

static const char *SOCK_PATH = "/tmp/spoofd.sock";

typedef pid_t (*getpid_t)(void);
typedef pid_t (*getppid_t)(void);
static getpid_t real_getpid;
static getppid_t real_getppid;

static void init_real_funcs() {
    if (!real_getpid) real_getpid = (getpid_t)dlsym(RTLD_NEXT, "getpid");
    if (!real_getppid) real_getppid = (getppid_t)dlsym(RTLD_NEXT, "getppid");
}

static int query_spoof(pid_t pid, pid_t *fake_pid, pid_t *fake_ppid) {
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return -1;
    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, SOCK_PATH, sizeof(addr.sun_path)-1);
    if (connect(fd, (struct sockaddr*)&addr, sizeof(addr)) < 0) { close(fd); return -1; }

    char buf[64];
    int len = snprintf(buf, sizeof(buf), "{\"pid\":%d}", pid);
    write(fd, buf, len);

    char resp[256] = {0};
    int n = read(fd, resp, sizeof(resp)-1);
    close(fd);
    if (n <= 0) return -1;

    char *p1 = strstr(resp, "\"fake_pid\":");
    char *p2 = strstr(resp, "\"fake_ppid\":");
    if (p1) *fake_pid = atoi(p1+11);
    if (p2) *fake_ppid = atoi(p2+12);
    return 0;
}

pid_t getpid(void) {
    init_real_funcs();
    pid_t real = real_getpid();
    pid_t fakep=0, fakepp=0;
    if (query_spoof(real, &fakep, &fakepp)==0 && fakep>0) return fakep;
    return real;
}

pid_t getppid(void) {
    init_real_funcs();
    pid_t real = real_getppid();
    pid_t fakep=0, fakepp=0;
    if (query_spoof(real, &fakep, &fakepp)==0 && fakepp>0) return fakepp;
    return real;
}