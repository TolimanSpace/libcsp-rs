#include <csp/csp.h>
#include <csp/interfaces/csp_if_zmqhub.h>
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

int main(void) {
    csp_conf_t conf;
    csp_conf_get_defaults(&conf);
    conf.address = 2;
    csp_init(&conf);

    csp_iface_t * zmq_if = NULL;
    int res = csp_zmqhub_init(2, "localhost", 0, &zmq_if);
    if (res != CSP_ERR_NONE) {
        fprintf(stderr, "Failed to init ZMQ hub\n");
        return 1;
    }
    csp_rtable_set(CSP_DEFAULT_ROUTE, 0, zmq_if, CSP_NO_VIA_ADDRESS);

    printf("C server started on address 2\n");

    csp_bind(NULL, 10);
    csp_listen(NULL, 5);

    printf("C server listening on port 10\n");

    csp_conn_t *conn = csp_accept(NULL, 10000); // 10s timeout
    if (conn) {
        printf("C server accepted connection\n");
        csp_packet_t *packet = csp_read(conn, 5000);
        if (packet) {
            printf("Received: %s\n", (char *)packet->data);
            csp_buffer_free(packet);
        }
        csp_close(conn);
    } else {
        printf("C server accept timeout\n");
        return 1;
    }

    return 0;
}
