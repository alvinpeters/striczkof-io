PORTNAME=		striczkof-io
PORTVERSION=	0.1.99
PORTREVISION=	1
CATEGORIES=		www
DISTNAME=		striczkof-io

PKGNAMESUFFIX=	-devel
MASTER_SITES=	GHC
GH_ACCOUNT=		striczkof

MAINTAINER=		alvin@striczkof.io
COMMENT=		My pathetic attempt of making my own web server + website with mostly Rust stuff.
WWW=			https://striczkof.io

LICENSE=		AGPLv3+
LICENSE_FILE=	${WRKSRC}/LICENSE

BUILD_DEPENDS=	rust:lang/rust

build:
	${WRKSRC}/build.sh

.include <bsd.port.mk>
