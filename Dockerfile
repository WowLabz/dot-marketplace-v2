FROM paritytech/ci-linux:production

WORKDIR /var/www/dot-marketplace-v2
COPY . /var/www/dot-marketplace-v2
EXPOSE 9944
