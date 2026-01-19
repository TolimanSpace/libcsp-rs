#include <csp/csp.h>
#include <csp/interfaces/csp_if_zmqhub.h>
#include <stdio.h>
#include <unistd.h>
#include <pthread.h>

void * router_task(void * param) {
    while (1) {
        csp_route_work();
    }
    return NULL;
}

int main(void) {
    csp_conf.address = 10;
    csp_init();
    csp_buffer_init();

    csp_iface_t *zmq_if;
    int res = csp_zmqhub_init_w_endpoints(10, "tcp://127.0.0.1:6000", "tcp://127.0.0.1:7000", 0, &zmq_if);
    if (res != CSP_ERR_NONE) {
        return 1;
    }
    
    csp_rtable_set(0, 0, zmq_if, CSP_NO_VIA_ADDRESS);

    pthread_t router;
    pthread_create(&router, NULL, router_task, NULL);

    static csp_socket_t sock;
    csp_bind(&sock, 10);
    csp_listen(&sock, 5);

    csp_conn_t *conn = csp_accept(&sock, 10000); 
    if (conn) {
        csp_packet_t *packet = csp_read(conn, 5000);
        if (packet) {
            printf("Received: %s\n", (char *)packet->data);
            csp_buffer_free(packet);
        }
        csp_close(conn);
    }

    return 0;
}
