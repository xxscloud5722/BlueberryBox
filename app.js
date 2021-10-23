const express = require('express');
const path = require('path');
const http = require('http');
const process = require('process');
const fs = require("fs");
const cheerio = require('cheerio');

const isCacheServer = process.argv.find(r => r === '--cache') != null;

// Default App
const app = express();

// Root Path
const root = process.cwd();

// Local Path
const localPath = path.join(root, 'web');
const localPathLength = localPath.length;


// Generate
const generateMetas = (seoConfig, requestPath) => {
    let metas = '';
    for (let config of seoConfig) {
        // Pre Match
        let flag = (config.preMatch != null && config.preMatch.length > 0 && requestPath.startsWith(config.preMatch));
        // Tail Match
        flag = flag || (config.tailMatch != null && config.tailMatch.length > 0 && requestPath.endsWith(config.tailMatch));
        // RegExp Match
        flag = flag || new RegExp(config.regexp).test(requestPath);
        // Non Match
        if (!flag) {
            continue;
        }
        for (let row of config.head) {
            let meta = '';
            for (let key in row) {
                if (row.hasOwnProperty(key)) {
                    meta += ` ${key}="${row[key]}"`;
                }
            }
            metas += `<meta ${meta}/>\n`;
        }
    }
    return metas;
}

// Async Read File Info
const getFileInfo = async (requestPath) => {
    return new Promise((resolve) => {
        fs.exists(requestPath, e => {
            if (!e) {
                return resolve(null);
            }
            fs.stat(requestPath, (err, s) => {
                if (!s.isFile()) {
                    return resolve(null);
                }
                return resolve(s);
            });
        })
    });
}

// Async Read File Data
const readFile = async (localPath) => {
    return new Promise((resolve) => {
        fs.readFile(localPath, (err, data) => {
            if (err != null) {
                return resolve(null);
            }
            resolve(data.toString('utf-8'));
        });
    });
}

// Cache Server
const serverCache = () => {
    // Web File Checklist
    const webChecklist = [];
    // Default Index.html
    let indexBuffer = null;
    // SEO File
    let seoConfig = []
    if (fs.existsSync(localPath) && fs.statSync(localPath).isDirectory()) {
        const scanDir = (scanPath) => {
            fs.readdirSync(scanPath).forEach(it => {
                const tempPath = path.join(scanPath, it);
                const fileInfo = fs.statSync(tempPath);
                if (fileInfo.isFile()) {
                    webChecklist.push(tempPath.substring(localPathLength));
                } else if (fileInfo.isDirectory()) {
                    scanDir(tempPath);
                }
            });
        };
        scanDir(localPath);


        const indexPath = path.join(localPath, 'index.html');
        if (fs.existsSync(indexPath) && fs.statSync(indexPath).isFile()) {
            indexBuffer = fs.readFileSync(indexPath);
        }

        const seoPath = path.join(localPath, 'seo.json');
        if (fs.existsSync(seoPath) && fs.statSync(seoPath).isFile()) {
            seoConfig = JSON.parse(fs.readFileSync(seoPath, 'utf-8'));
            if (!(seoConfig instanceof Array)) {
                throw 'seo.json error';
            }
        }
    }

    app.get("/**", (request, response) => {
        if (webChecklist.indexOf(request.path) > -1) {
            response.sendFile(path.join(localPath, request.path));
            return;
        }
        if (indexBuffer == null || indexBuffer.length <= 0) {
            response.end();
            return;
        }
        const index = cheerio.load(indexBuffer.toString('utf-8'));
        index('head').prepend(generateMetas(seoConfig, request.path));

        response.send(index.html());
    });
}

// Non Server
const serverNonCache = () => {
    app.get("/**", async (request, response) => {
        const serverPath = path.join(localPath, request.path);
        const fileInfo = await getFileInfo(serverPath);
        if (fileInfo != null) {
            return response.sendFile(serverPath);
        }

        const indexPath = path.join(localPath, 'index.html');
        const indexInfo = await getFileInfo(indexPath);
        if (indexInfo != null && indexInfo.isFile()) {
            const seoPath = path.join(localPath, 'seo.json');
            const seoInfo = await getFileInfo(seoPath);
            if (seoInfo != null && seoInfo.isFile()) {
                const seoConfig = JSON.parse((await readFile(seoPath)) || '[]');
                if (!(seoConfig instanceof Array)) {
                    throw 'seo.json error';
                }
                if (seoConfig.length > 0) {
                    const index = cheerio.load(await readFile(indexPath));
                    index('head').prepend(generateMetas(seoConfig, request.path));
                    return response.send(index.html());
                }
            }
            return response.sendFile(indexPath);
        }

        return response.end();
    });
}

if (isCacheServer) {
    console.log('Model: ServerCache');
    serverCache();
} else {
    console.log('Model: ServerNonCache');
    serverNonCache();
}


// Start Server
const port = process.env.PORT || '3000';
const server = http.createServer(app);
server.listen(port);
server.on('error', (error) => {
    if (error.syscall !== 'listen') {
        throw error;
    }

    const bind = typeof port === 'string'
        ? 'Pipe ' + port
        : 'Port ' + port;

    // handle specific listen errors with friendly messages
    switch (error.code) {
        case 'EACCES':
            console.error(bind + ' requires elevated privileges');
            process.exit(1);
            break;
        case 'EADDRINUSE':
            console.error(bind + ' is already in use');
            process.exit(1);
            break;
        default:
            throw error;
    }
});
server.on('listening', () => {
    const address = server.address();
    const bind = typeof address === 'string'
        ? 'pipe ' + address
        : 'port ' + address.port;
    console.log('Listening on ' + bind);
});


