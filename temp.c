#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <unistd.h>
#include <getopt.h>
#include <signal.h>
#include <errno.h>
#include <ftdi.h>

#define VID 0x0403
#define PID 0x6010
#define CHUNKSZ 8000000
#define MAGIC 8272



int main(int argc, char **argv) {
    // check argument numbers
    if(argc < 3) {
        fprintf(stderr, "Usage: ./lvds-xfer <filename> <transfer size in hex>\n");
        return EXIT_FAILURE;
    }

    struct ftdi_context *ftdi;

    // open FTDI device A in synchronous 245 mode. This device is used for data transfer
    if ((ftdi = ftdi_new()) == 0) {
        fprintf(stderr, "ftdi_new failed on channel A\n");
        
        return EXIT_FAILURE;
    }

    if (ftdi_set_interface(ftdi, INTERFACE_A) < 0) {
        fprintf(stderr, "ftdi_set_interface  for device A failed\n");
        ftdi_free(ftdi);
        
        return EXIT_FAILURE;
    }

    if (ftdi_usb_open(ftdi, VID, PID) < 0) {
        fprintf(stderr,"Can't open ftdi device A: %s\n",ftdi_get_error_string(ftdi));
        ftdi_free(ftdi);
        
        return EXIT_FAILURE;
    }

    if (ftdi_set_bitmode(ftdi,  0xFF, BITMODE_SYNCFF) < 0) {
        fprintf(stderr,"Can't set synchronous fifo mode on device A, Error %s\n",ftdi_get_error_string(ftdi));
        ftdi_usb_close(ftdi);
        ftdi_free(ftdi);
        
        return EXIT_FAILURE;
    }

    /* A timeout value of 1 results in may skipped blocks */
    /* The timeout sets the period where the ftdi->computer buffer is transmitted, not needed for tx-ing */
    if(ftdi_set_latency_timer(ftdi, 2)) {
        fprintf(stderr,"Can't set latency on device A, Error %s\n",ftdi_get_error_string(ftdi));
        ftdi_usb_close(ftdi);
        ftdi_free(ftdi);

        return EXIT_FAILURE;
    }

    /* ???equiv of FT_SetUSBParameters()??? */
    if (ftdi_write_data_set_chunksize(ftdi, CHUNKSZ) != 0) {
        fprintf(stderr,"Can't set chunk size on device A, Error %s\n",ftdi_get_error_string(ftdi));
        ftdi_usb_close(ftdi);
        ftdi_free(ftdi);
        
        return EXIT_FAILURE;
    }

    if (ftdi_setflowctrl(ftdi, SIO_RTS_CTS_HS) != 0) {
        fprintf(stderr,"Can't set flow control on device A, Error %s\n",ftdi_get_error_string(ftdi));
        ftdi_usb_close(ftdi);
        ftdi_free(ftdi);
        
        return EXIT_FAILURE;
    }

    // Open FTDI device B in asynchronous bitbang mode. This device is used to configure packet size
    // on channel A
    // ERROR HANDLING NOTE: If this interface cannot be opened, data transfers can still be run
    // this doesn't work yet?
    /*
    struct ftdi_context *ftdib;
    if ((ftdib = ftdi_new()) == 0) {
        fprintf(stderr, "ftdi_new failed on channel B\n");

        goto skip_b_config;
    }

    if (ftdi_set_interface(ftdib, INTERFACE_B) < 0) {
        fprintf(stderr, "ftdi_set_interface  for device B failed\n");
        ftdi_free(ftdib);

        goto skip_b_config;
    }

    if (ftdi_usb_open(ftdib, VID, PID) < 0) {
        fprintf(stderr,"Can't open ftdi device B: %s\n",ftdi_get_error_string(ftdib));
        ftdi_free(ftdib);
        
        goto skip_b_config;
    }

    if (ftdi_set_bitmode(ftdib,  0xFF, BITMODE_BITBANG) < 0) {
        fprintf(stderr,"Can't set async bitbang on channel B, Error %s\n",ftdi_get_error_string(ftdib));
        ftdi_usb_close(ftdib);
        ftdi_free(ftdib);
        
        goto skip_b_config;
    }

    // configure packet size
    unsigned char packetSize[1]; 
    packetSize[0] = strtol(argv[2], NULL, 16);
    if (ftdi_write_data(ftdib, packetSize, 1) < 0) {
        fprintf(stderr,"Can't write to channel B, Error %s\n",ftdi_get_error_string(ftdib));
    }
    */
    
skip_b_config: ;
    // open a file and xfer it
    unsigned char buffer[CHUNKSZ];
    FILE *fptr;

    fptr = fopen(argv[1], "rb");
    if (fptr == NULL) {
        printf("unable to read file\nPWD: ");
        system("pwd");

        return EXIT_FAILURE;
    }

    int64_t freadout, writeTotal = 0;
    int transferable = 1;
    int xtra = 0;

    while(transferable) {
        freadout = fread(buffer, 1, CHUNKSZ, fptr);
        if (freadout != CHUNKSZ) {
            //printf("fread cannot read file or enough bits. Only read %d, EOF?\n", freadout);
            transferable = 0;
        }

	    writeTotal += freadout;
        ftdi_write_data(ftdi, buffer, freadout);
        //printf("written bits %d, total written %d\n", ftdi_write_data(ftdi, buffer, freadout), writeTotal);
    }

    unsigned char array[8272] = {'0'};
    xtra = writeTotal % MAGIC;
    ftdi_write_data(ftdi, array ,xtra);

    printf("written bytes: %lld + padding %d\n", writeTotal, xtra);

    ftdi_free(ftdi);
    ftdi_free(ftdib);

    return 0;
}
