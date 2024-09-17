Install

------

(should work as is on your mac, all the settings should work already with the code)

tar xvf prometheus.tar.gz
cd prometheus
./prometheus


-------

grafana if not already in local

brew install grafana
brew service start grafana

(admin, admin are default user/pswd)

-------
connect prometheus as a data source
new dashboard from import, use the file in this repo

----

start the test, update the values in src/main.rs if necessary

install rust if not already:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh


run the app in release mode:

cargo run -r
