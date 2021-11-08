const express = require('express');
const path = require('path');
const http = require('http');
const process = require('process');
const fs = require("fs");
const cheerio = require('cheerio');
const chokidar = require('chokidar');
const cluster = require('cluster');

// async utils
const utils = {
    /**
     * Ready file info.
     * @param path File Path.
     * @returns {Promise} Async file info or null.
     */
    getFileInfo: async (path) => {
        return new Promise((resolve) => {
            fs.exists(path, result => {
                if (!result) {
                    return resolve(null);
                }
                fs.stat(path, (error, stats) => {
                    if (error != null) {
                        return resolve(null);
                    }
                    if (!stats.isFile()) {
                        return resolve(null);
                    }
                    return resolve(stats);
                });
            })
        });
    },

    /**
     * Read file data.
     * @param localPath File Path.
     * @returns {Promise} Async file data or null.
     */
    readFile: async (localPath) => {
        return new Promise((resolve) => {
            fs.readFile(localPath, (error, data) => {
                if (error != null) {
                    return resolve(null);
                }
                resolve(data.toString('utf-8'));
            });
        });
    },

    /**
     * Read Dir info.
     * @param localPath File path.
     * @returns {Promise} Async dir info or null.
     */
    readDir: async (localPath) => {
        return new Promise(resolve => {
            fs.readdir(localPath, (error, files) => {
                if (error != null) {
                    return resolve(null);
                }
                resolve(files);
            });
        });
    }
};

// resource class
class Resource {
    files = [];
    index = null;
    seo = null;
    suffix = ['favicon.ico'];

    async scanDir(localPath, prefixPathLength) {
        const fileList = await utils.readDir(localPath);
        if (fileList != null && fileList.length > 0) {
            for (const it of fileList) {
                const tempPath = path.join(localPath, it);
                const fileInfo = await utils.getFileInfo(tempPath);
                if (fileInfo == null) {
                    continue;
                }
                if (fileInfo.isFile()) {
                    await this.loadFile(tempPath, prefixPathLength);
                } else if (fileInfo.isDirectory()) {
                    await this.scanDir(tempPath, prefixPathLength);
                }
            }
        }
    }

    async addFile(path, prefixPathLength) {
        const fileInfo = await utils.getFileInfo(path);
        if (fileInfo == null) {
            return;
        }
        if (fileInfo.isFile()) {
            // is index.html
            if (path.endsWith("index.html")) {
                this.index = await utils.readFile(path) || '';
            }
            // is seo.json
            if (path.endsWith("seo.json")) {
                const seoJson = await utils.readFile(path);
                if (seoJson != null) {
                    this.seo = JSON.parse(seoJson);
                    if (!(this.seo instanceof Array)) {
                        throw 'seo.json error';
                    }
                }
            }
            this.files.push(path.substring(prefixPathLength));
        }
    }

    async removeFile(path) {
        const index = this.files.findIndex(r => path.endsWith(r));
        this.files.splice(index, 1);
    }

    async match(path) {
        if (this.suffix.findIndex(r => path.toLowerCase().endsWith(r)) > -1) {
            return 2;
        }
        return this.files.findIndex(r => r.startsWith(path)) > -1 ? 1 : 0;
    }

    async existIndex() {
        return this.index != null;
    }

    async getIndexData() {
        return this.index || '';
    }

    async generateMetas(requestPath) {
        let metas = '';
        for (let config of (this.seo || [])) {
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
}

// main
async function main() {
    // Default App
    const app = express();

    // Root Path
    const root = process.cwd();

    // Local Path
    const localPath = path.join(root, process.env.LOCAL_PATH || 'web');
    console.log('LocalPath: ' + localPath);

    // Resource.
    const resource = new Resource();

    // Resource Watch
    chokidar.watch(localPath).on('all', async (event, path) => {
        if (event === 'unlink') {
            await resource.removeFile(path);
        } else if (event === 'add') {
            await resource.addFile(path, localPath.length);
        }
    });

    // Request
    app.get("/**", async (request, response) => {
        const status = await resource.match(request.path);
        if (status === 1) {
            response.sendFile(path.join(localPath, request.path));
            return;
        }

        if (status === 2) {
            response.end();
            return;
        }

        if (!(await resource.existIndex())) {
            response.end();
            return;
        }

        const metas = await resource.generateMetas(request.path);
        if (metas == null || metas.length <= 0) {
            response.send(await resource.getIndexData());
            return;
        }

        const index = cheerio.load(await resource.getIndexData());
        index('head').prepend(metas);
        response.send(index.html());
    });

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
}

if (cluster.isMaster) {
    const numCPUs = require('os').cpus().length;
    // Fork workers
    for (let i = 0; i < numCPUs; i++) {
        cluster.fork();
    }

    cluster.on('exit', function (worker, code, signal) {
        console.log('worker ' + worker.process.pid + ' restart');
        setTimeout(function () {
            cluster.fork();
        }, 2000);
    });
} else {
    main().then();
}


