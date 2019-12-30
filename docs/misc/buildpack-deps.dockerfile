FROM buildpack-deps:eoan

RUN set -ex; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
    vim \
    flex \
    bison \
    \    
    && rm -rf /var/lib/apt/lists/* 
WORKDIR /opt
RUN wget http://archive.openwrt.org/chaos_calmer/15.05/ramips/mt7621/OpenWrt-SDK-15.05-ramips-mt7621_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64.tar.bz2 ; \
    mkdir openwrt-sdk; \
    tar xjf OpenWrt-SDK-15.05-ramips-mt7621_gcc-4.8-linaro_uClibc-0.9.33.2.Linux-x86_64.tar.bz2 --strip-components=1 -C openwrt-sdk
ENV PATH=/opt/openwrt-sdk/staging_dir/toolchain-mipsel_1004kc+dsp_gcc-4.8-linaro_uClibc-0.9.33.2/bin:${PATH} \
    STAGING_DIR=/opt/openwrt-sdk
