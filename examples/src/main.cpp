
#include<iostream>
#include<opencv2/opencv.hpp>

using namespace std;
using namespace cv;

int main(){
    Mat frame;
    VideoCapture cap(0);
    for(;;){
   	cap >> frame;
        imshow("frame", frame);
	waitKey(20);
    }
    cout << "the code was updated" << endl;
    return 0;
}
