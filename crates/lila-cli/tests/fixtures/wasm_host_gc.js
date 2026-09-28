var result = gc();
if (result !== undefined) throw new Error('gc must return undefined');
print('gc returned undefined');
