FROM paritytech/ci-linux:production

WORKDIR /var/www/dot_marketplace_v2
COPY . /var/www/dot_marketplace_v2
EXPOSE 9944
