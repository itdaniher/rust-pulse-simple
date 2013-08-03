all:
	rustc --opt-level=3 --link-args '-lkissfft -lpulse -lpulse-simple' -L./ patest.rc

clean:
	rm patest
