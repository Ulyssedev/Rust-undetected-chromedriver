FROM debian:bullseye-slim
RUN useradd apps
RUN mkdir -p /home/apps && chown apps:apps /home/apps
COPY ./xvfb.sh /home/apps/xvfb.sh
RUN chmod +x /home/apps/xvfb.sh
RUN apt-get update && apt-get install -y xvfb fluxbox wget wmctrl gnupg2 libc6-dev libasound2 fonts-liberation libappindicator3-1 libatk-bridge2.0-0 libatk1.0-0 libatspi2.0-0 libcairo2 libcups2 libdbus-1-3 libdrm2 libgbm1 libgdk-pixbuf2.0-0 libgtk-3-0 libnspr4 libnss3 libpango-1.0-0 libx11-xcb1 libxcb-dri3-0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libxshmfence1 libxss1 libxtst6 lsb-release xdg-utils libu2f-udev && rm -rf /var/lib/apt/lists/*
RUN wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb && dpkg -i google-chrome-stable_current_amd64.deb && rm google-chrome-stable_current_amd64.deb
USER apps
WORKDIR /home/apps
ENTRYPOINT ["bash", "/home/apps/xvfb.sh"]
