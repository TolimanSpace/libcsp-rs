#include <zmq.h>
#include <stdio.h>

int main(void) {
    void *context = zmq_ctx_new();
    void *xpub = zmq_socket(context, ZMQ_XPUB);
    void *xsub = zmq_socket(context, ZMQ_XSUB);
    
    int res;
    res = zmq_bind(xsub, "tcp://*:6000");
    if (res != 0) { printf("Failed to bind xsub: %d\n", res); return 1; }
    res = zmq_bind(xpub, "tcp://*:7000");
    if (res != 0) { printf("Failed to bind xpub: %d\n", res); return 1; }
    
    printf("ZMQ Hub started on 6000(XSUB)/7000(XPUB)\n");
    fflush(stdout);
    
    zmq_proxy(xpub, xsub, NULL);
    
    zmq_close(xpub);
    zmq_close(xsub);
    zmq_ctx_destroy(context);
    return 0;
}
