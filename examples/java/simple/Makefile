JAVAC = javac
JAVA = java
OUT = out

.PHONY: all test clean

all:
	mkdir -p $(OUT)
	$(JAVAC) -d $(OUT) src/MdisClient.java Example1.java Example2.java Example3.java Example4.java

test:
	mkdir -p $(OUT)
	$(JAVAC) -d $(OUT) src/MdisClient.java test/MdisClientTest.java
	$(JAVA) -cp $(OUT) MdisClientTest

clean:
	rm -rf $(OUT)
