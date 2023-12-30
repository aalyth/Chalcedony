.PHONY: release syntax clean

release: ./src/main.rs
	if [ -z $(shell which cargo) ]; then \
		echo 'Error: you need to have cargo installed'; \
		exit 1; \
	fi
	if [ $(shell id -u) -ne 0 ]; then \
		echo 'Error: you need to run the script as sudo in order to place the executable in /usr/local/bin'; \
		exit 1; \
	fi 
	cargo build --release
	cp ./target/release/chalcedony /usr/local/bin/chal

syntax: ./utils/syntax/chal.vim
	# adds syntax highlighting for *.ch files
	# for vim:
	mkdir -p ~/.vim/syntax 
	cp ./utils/syntax/chal.vim ~/.vim/syntax/ch.vim
	mkdir -p ~/.vim/ftdetect
	touch ~/.vim/ftdetect/ch.vim
	echo "au BufRead, BufNewFile *.ch setfiletype chalcedony" > ~/.vim/ftdetect/chl.vim
	# for nvim:
	mkdir -p ~/.config/nvim/syntax 
	cp ./utils/syntax/chal.vim ~/.config/nvim/syntax/ch.vim

clean:
	cargo clean